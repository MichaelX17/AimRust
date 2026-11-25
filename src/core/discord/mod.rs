//! Discord Rich Presence integration for AimRust.
//!
//! Runs Discord Local IPC on a dedicated background thread so the main render/input loop
//! never stalls. Configure your Discord Developer Portal Application ID via:
//! 1. `AIMRUST_DISCORD_APP_ID` environment variable,
//! 2. `discord_app_id.txt` inside the AimRust config directory, or
//! 3. `DEFAULT_DISCORD_APP_ID` below.

mod ipc;

use std::sync::mpsc::{self, RecvTimeoutError, Sender};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use bevy::prelude::*;

use crate::core::paths;
use crate::core::round::RoundTimer;
use crate::core::score::{RoundScore, click_accuracy, scoring_mode};
use crate::core::state::AppState;
use crate::core::task_api::{ScoringMode, SelectedTask, TaskRegistry};

use ipc::{ActivityPayload, DiscordIpcClient};

/// Discord Developer Portal Application ID for AimRust Rich Presence.
pub const DEFAULT_DISCORD_APP_ID: &str = "1575133841885102090";

/// Rich Presence Art Asset key uploaded in the Discord Developer Portal (`aim-rust.png`).
pub const LARGE_IMAGE_KEY: &str = "aim_rust_logo";
pub const LARGE_IMAGE_TEXT: &str = "AimRust";

const RECONNECT_INTERVAL: Duration = Duration::from_secs(10);

pub struct DiscordPresencePlugin;

impl Plugin for DiscordPresencePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, init_discord_presence).add_systems(
            Update,
            sync_discord_presence.run_if(resource_exists::<DiscordPresence>),
        );
    }
}

#[derive(Resource)]
struct DiscordPresence {
    tx: Sender<ActivityPayload>,
    session_start: u64,
    last_sent: Option<ActivityPayload>,
}

fn unix_now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn resolve_client_id() -> Option<String> {
    if let Ok(env_id) = std::env::var("AIMRUST_DISCORD_APP_ID") {
        let trimmed = env_id.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_owned());
        }
    }

    if let Some(path) = paths::config_file_path("discord_app_id.txt")
        && let Ok(contents) = std::fs::read_to_string(path)
    {
        let trimmed = contents.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_owned());
        }
    }

    let default_id = DEFAULT_DISCORD_APP_ID.trim();
    if !default_id.is_empty() {
        return Some(default_id.to_owned());
    }

    None
}

fn init_discord_presence(mut commands: Commands) {
    let Some(client_id) = resolve_client_id() else {
        info!(
            "Discord Rich Presence inactive: set DEFAULT_DISCORD_APP_ID, AIMRUST_DISCORD_APP_ID, or discord_app_id.txt"
        );
        return;
    };

    let (tx, rx) = mpsc::channel::<ActivityPayload>();
    let spawn_result = std::thread::Builder::new()
        .name("aimrust-discord-rpc".into())
        .spawn(move || rpc_worker(client_id, rx));

    if let Err(err) = spawn_result {
        warn!("Failed to spawn Discord RPC worker thread: {err}");
        return;
    }

    let session_start = unix_now_secs();
    let initial = ActivityPayload {
        details: "In Main Menu".into(),
        state: "Ready to train".into(),
        start_timestamp: Some(session_start),
        end_timestamp: None,
        large_image: LARGE_IMAGE_KEY,
        large_text: LARGE_IMAGE_TEXT,
    };
    let _ = tx.send(initial.clone());

    commands.insert_resource(DiscordPresence {
        tx,
        session_start,
        last_sent: Some(initial),
    });
}

fn rpc_worker(client_id: String, rx: mpsc::Receiver<ActivityPayload>) {
    let mut client: Option<DiscordIpcClient> = None;
    let mut latest_activity: Option<ActivityPayload> = None;

    loop {
        match rx.recv_timeout(RECONNECT_INTERVAL) {
            Ok(mut activity) => {
                // Coalesce bursty state transitions to the most recent snapshot.
                while let Ok(newer) = rx.try_recv() {
                    activity = newer;
                }
                latest_activity = Some(activity);
                push_activity(&client_id, &mut client, latest_activity.as_ref());
            }
            Err(RecvTimeoutError::Timeout) => {
                // Periodically retry connecting if Discord started after AimRust.
                if client.is_none() && latest_activity.is_some() {
                    push_activity(&client_id, &mut client, latest_activity.as_ref());
                }
            }
            Err(RecvTimeoutError::Disconnected) => break,
        }
    }

    if let Some(active) = client {
        active.close();
    }
}

fn push_activity(
    client_id: &str,
    client: &mut Option<DiscordIpcClient>,
    activity: Option<&ActivityPayload>,
) {
    let Some(activity) = activity else {
        return;
    };

    if client.is_none() {
        match DiscordIpcClient::connect(client_id) {
            Ok(connected) => *client = Some(connected),
            Err(_) => return,
        }
    }

    if let Some(conn) = client.as_mut()
        && conn.set_activity(activity).is_err()
    {
        *client = None;
    }
}

fn task_display_name<'a>(registry: &'a TaskRegistry, selected: &'a SelectedTask) -> &'a str {
    registry
        .0
        .iter()
        .find(|t| t.id == selected.0)
        .map(|t| t.name)
        .filter(|name| !name.is_empty())
        .unwrap_or("Training")
}

fn sync_discord_presence(
    state: Res<State<AppState>>,
    selected: Res<SelectedTask>,
    registry: Res<TaskRegistry>,
    timer: Res<RoundTimer>,
    score: Res<RoundScore>,
    mut presence: ResMut<DiscordPresence>,
) {
    if !state.is_changed() && !selected.is_changed() {
        return;
    }

    let task_name = task_display_name(&registry, &selected);
    let now = unix_now_secs();

    let (details, state_text, start_timestamp, end_timestamp) = match *state.get() {
        AppState::Menu => (
            "In Main Menu".to_owned(),
            "Ready to train".to_owned(),
            Some(presence.session_start),
            None,
        ),
        AppState::Settings => (
            "Configuring Settings".to_owned(),
            "Customizing loadout".to_owned(),
            Some(presence.session_start),
            None,
        ),
        AppState::TaskSelect => (
            "Browsing Tasks".to_owned(),
            "Choosing a scenario".to_owned(),
            Some(presence.session_start),
            None,
        ),
        AppState::TimeSelect => (
            format!("Preparing {task_name}"),
            "Selecting round duration".to_owned(),
            Some(presence.session_start),
            None,
        ),
        AppState::Countdown => (
            format!("Starting {task_name}"),
            "Get ready...".to_owned(),
            Some(presence.session_start),
            None,
        ),
        AppState::Playing => {
            let remaining = timer.remaining.ceil().max(0.0) as u64;
            let end = (remaining > 0).then_some(now.saturating_add(remaining));
            let start = end.is_none().then_some(now);
            (format!("Playing {task_name}"), "In Round".to_owned(), start, end)
        }
        AppState::Paused => (
            format!("Paused — {task_name}"),
            "Taking a breather".to_owned(),
            None,
            None,
        ),
        AppState::Results => {
            let summary = if timer.doa_failed {
                "DOA Failed".to_owned()
            } else if scoring_mode(&registry, &selected) == ScoringMode::Click {
                let acc = click_accuracy(&score);
                format!("Score: {} • {acc:.0}% acc", score.points)
            } else {
                format!("Score: {}", score.points)
            };
            (
                format!("Finished {task_name}"),
                summary,
                Some(presence.session_start),
                None,
            )
        }
    };

    let next = ActivityPayload {
        details,
        state: state_text,
        start_timestamp,
        end_timestamp,
        large_image: LARGE_IMAGE_KEY,
        large_text: LARGE_IMAGE_TEXT,
    };

    if presence.last_sent.as_ref() == Some(&next) {
        return;
    }

    if presence.tx.send(next.clone()).is_ok() {
        presence.last_sent = Some(next);
    }
}
