use std::time::{Duration, Instant};

use bevy::prelude::*;
use bevy::window::{
    MonitorSelection, PresentMode, PrimaryWindow, VideoModeSelection,
    WindowMode as BevyWindowMode,
};

use crate::core::config::{Settings, WindowMode};

/// Applies vsync/window mode when settings change and enforces the fps cap.
pub fn apply_video_settings(
    settings: Res<Settings>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    mut last_frame: Local<Option<Instant>>,
) {
    if settings.is_changed()
        && let Some(mut window) = windows.iter_mut().next()
    {
            window.present_mode = if settings.video.vsync {
                PresentMode::AutoVsync
            } else {
                PresentMode::AutoNoVsync
            };
            window.mode = match settings.video.window_mode {
                WindowMode::Windowed => BevyWindowMode::Windowed,
                WindowMode::Borderless => {
                    BevyWindowMode::BorderlessFullscreen(MonitorSelection::Current)
                }
                WindowMode::Fullscreen => {
                    // Wayland does not support exclusive fullscreen (`winit` ignores it);
                    // compositor fullscreen (`BorderlessFullscreen`) is the native equivalent.
                    if is_wayland_session() {
                        BevyWindowMode::BorderlessFullscreen(MonitorSelection::Current)
                    } else {
                        BevyWindowMode::Fullscreen(
                            MonitorSelection::Current,
                            VideoModeSelection::Current,
                        )
                    }
                }
            };
    }

    let cap = settings.video.fps_cap;
    if cap > 0 {
        let target = Duration::from_secs_f64(1.0 / cap as f64);
        if let Some(previous) = *last_frame {
            let elapsed = previous.elapsed();
            if elapsed < target {
                std::thread::sleep(target - elapsed);
            }
        }
    }
    *last_frame = Some(Instant::now());
}

fn is_wayland_session() -> bool {
    #[cfg(target_os = "linux")]
    {
        std::env::var_os("WAYLAND_DISPLAY").is_some()
    }
    #[cfg(not(target_os = "linux"))]
    {
        false
    }
}
