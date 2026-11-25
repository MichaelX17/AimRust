//! Minimal zero-dependency Discord Local IPC transport (Unix domain sockets + Windows named pipes).

use std::io::{self, Read, Write};
#[cfg(unix)]
use std::os::unix::net::UnixStream;
#[cfg(unix)]
use std::path::PathBuf;
#[cfg(unix)]
use std::time::Duration;

const OP_HANDSHAKE: u32 = 0;
const OP_FRAME: u32 = 1;
const OP_CLOSE: u32 = 2;
const OP_PING: u32 = 3;
const OP_PONG: u32 = 4;

const MAX_FRAME_LEN: usize = 1024 * 1024;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ActivityPayload {
    pub details: String,
    pub state: String,
    pub start_timestamp: Option<u64>,
    pub end_timestamp: Option<u64>,
    pub large_image: &'static str,
    pub large_text: &'static str,
}

#[cfg(unix)]
type Transport = UnixStream;

#[cfg(windows)]
type Transport = std::fs::File;

pub struct DiscordIpcClient {
    stream: Transport,
    nonce: u64,
}

impl DiscordIpcClient {
    /// Connects to a local Discord IPC socket/pipe and completes the v1 handshake.
    pub fn connect(client_id: &str) -> io::Result<Self> {
        let stream = open_transport()?;
        let mut client = Self { stream, nonce: 0 };

        let handshake = format!(r#"{{"v":1,"client_id":"{}"}}"#, escape_json(client_id));
        client.write_frame(OP_HANDSHAKE, handshake.as_bytes())?;

        let (op, body) = client.read_frame()?;
        if op == OP_CLOSE || body.contains(r#""evt":"ERROR""#) {
            return Err(io::Error::new(
                io::ErrorKind::ConnectionRefused,
                format!("Discord handshake rejected: {body}"),
            ));
        }

        Ok(client)
    }

    /// Sends a `SET_ACTIVITY` frame for the current process.
    pub fn set_activity(&mut self, activity: &ActivityPayload) -> io::Result<()> {
        self.nonce = self.nonce.wrapping_add(1);
        let payload = build_set_activity_json(std::process::id(), activity, self.nonce);
        self.write_frame(OP_FRAME, payload.as_bytes())?;
        self.drain_response()
    }

    /// Clears any active presence and closes the IPC session cleanly.
    pub fn close(mut self) {
        let _ = self.write_frame(OP_CLOSE, b"{}");
    }

    fn drain_response(&mut self) -> io::Result<()> {
        // Respond to any PING frames before returning on the activity reply.
        for _ in 0..4 {
            let (op, body) = self.read_frame()?;
            match op {
                OP_PING => {
                    self.write_frame(OP_PONG, body.as_bytes())?;
                }
                OP_CLOSE => {
                    return Err(io::Error::new(
                        io::ErrorKind::ConnectionReset,
                        "Discord closed IPC connection",
                    ));
                }
                _ => return Ok(()),
            }
        }
        Ok(())
    }

    fn write_frame(&mut self, opcode: u32, payload: &[u8]) -> io::Result<()> {
        let len = u32::try_from(payload.len())
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "IPC payload too large"))?;
        let mut header = [0u8; 8];
        header[0..4].copy_from_slice(&opcode.to_le_bytes());
        header[4..8].copy_from_slice(&len.to_le_bytes());

        self.stream.write_all(&header)?;
        self.stream.write_all(payload)?;
        self.stream.flush()
    }

    fn read_frame(&mut self) -> io::Result<(u32, String)> {
        let mut header = [0u8; 8];
        self.stream.read_exact(&mut header)?;
        let opcode = u32::from_le_bytes([header[0], header[1], header[2], header[3]]);
        let len = u32::from_le_bytes([header[4], header[5], header[6], header[7]]) as usize;
        if len > MAX_FRAME_LEN {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Discord IPC frame exceeded size limit",
            ));
        }

        let mut buf = vec![0u8; len];
        self.stream.read_exact(&mut buf)?;
        let body = String::from_utf8_lossy(&buf).into_owned();
        Ok((opcode, body))
    }
}

#[cfg(unix)]
fn open_transport() -> io::Result<Transport> {
    let mut dirs = Vec::new();
    if let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR")
        && !runtime_dir.is_empty()
    {
        let base = PathBuf::from(runtime_dir);
        dirs.push(base.clone());
        dirs.push(base.join("app/com.discordapp.Discord"));
        dirs.push(base.join("app/com.discordapp.DiscordCanary"));
        dirs.push(base.join("app/com.discordapp.DiscordPTB"));
        dirs.push(base.join("snap.discord"));
    }
    for var in ["TMPDIR", "TMP", "TEMP"] {
        if let Ok(val) = std::env::var(var)
            && !val.is_empty()
        {
            let path = PathBuf::from(val);
            if !dirs.contains(&path) {
                dirs.push(path);
            }
        }
    }
    let tmp = PathBuf::from("/tmp");
    if !dirs.contains(&tmp) {
        dirs.push(tmp);
    }

    for dir in dirs {
        for i in 0..10 {
            let candidate = dir.join(format!("discord-ipc-{i}"));
            if let Ok(stream) = UnixStream::connect(&candidate) {
                let timeout = Some(Duration::from_secs(2));
                let _ = stream.set_read_timeout(timeout);
                let _ = stream.set_write_timeout(timeout);
                return Ok(stream);
            }
        }
    }

    Err(io::Error::new(
        io::ErrorKind::NotFound,
        "No active Discord IPC socket found",
    ))
}

#[cfg(windows)]
fn open_transport() -> io::Result<Transport> {
    for i in 0..10 {
        let pipe = format!(r"\\.\pipe\discord-ipc-{i}");
        if let Ok(file) = std::fs::OpenOptions::new().read(true).write(true).open(&pipe) {
            return Ok(file);
        }
    }
    Err(io::Error::new(
        io::ErrorKind::NotFound,
        "No active Discord IPC pipe found",
    ))
}

#[cfg(not(any(unix, windows)))]
fn open_transport() -> io::Result<Transport> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "Discord IPC unsupported on this platform",
    ))
}

pub fn build_set_activity_json(pid: u32, activity: &ActivityPayload, nonce: u64) -> String {
    let mut fields = Vec::with_capacity(4);
    fields.push(format!(r#""details":"{}""#, escape_json(&activity.details)));
    fields.push(format!(r#""state":"{}""#, escape_json(&activity.state)));

    let mut ts_parts = Vec::with_capacity(2);
    if let Some(start) = activity.start_timestamp {
        ts_parts.push(format!(r#""start":{start}"#));
    }
    if let Some(end) = activity.end_timestamp {
        ts_parts.push(format!(r#""end":{end}"#));
    }
    if !ts_parts.is_empty() {
        fields.push(format!(r#""timestamps":{{{}}}"#, ts_parts.join(",")));
    }

    if !activity.large_image.is_empty() {
        fields.push(format!(
            r#""assets":{{"large_image":"{}","large_text":"{}"}}"#,
            escape_json(activity.large_image),
            escape_json(activity.large_text),
        ));
    }

    format!(
        r#"{{"cmd":"SET_ACTIVITY","args":{{"pid":{pid},"activity":{{{}}}}},"nonce":"aimrust-{nonce}"}}"#,
        fields.join(","),
    )
}

fn escape_json(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '"' => out.push_str(r#"\""#),
            '\\' => out.push_str(r"\\"),
            '\n' => out.push_str(r"\n"),
            '\r' => out.push_str(r"\r"),
            '\t' => out.push_str(r"\t"),
            c if (c as u32) < 0x20 => {
                use std::fmt::Write as _;
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_valid_activity_json() {
        let payload = ActivityPayload {
            details: "Playing \"Gridshot\"".into(),
            state: "Score: 120 • 95% acc".into(),
            start_timestamp: Some(1_700_000_000),
            end_timestamp: None,
            large_image: "aim_rust_logo",
            large_text: "AimRust",
        };
        let json = build_set_activity_json(42, &payload, 7);
        assert!(json.contains(r#""cmd":"SET_ACTIVITY""#));
        assert!(json.contains(r#""pid":42"#));
        assert!(json.contains(r#""details":"Playing \"Gridshot\"""#));
        assert!(json.contains(r#""start":1700000000"#));
        assert!(json.contains(r#""large_image":"aim_rust_logo""#));
        assert!(json.contains(r#""nonce":"aimrust-7""#));
    }
}
