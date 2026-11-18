Target Files: src/core/input.rs, src/core/mod.rs
Read-Only Files: none (PlayerCamera and PLAYER_POS are in Progress.md Interfaces)
Task_07: Mouse look, FOV, cursor, Shot

1. input.rs: `PlayerLook { yaw: f32, pitch: f32 }` resource; reset to 0 on OnEnter(Countdown).
2. Mouse look in Playing only: raw motion deltas; yaw/pitch change = delta * settings.sens * 0.022 degrees (const BASE_DEG_PER_COUNT = 0.022). Pitch clamp +-89. Yaw unclamped. Apply to PlayerCamera rotation (position stays PLAYER_POS).
3. FOV: convert Settings.fov_h to vertical using window aspect; update PlayerCamera projection on settings or window resize.
4. Cursor: grabbed + hidden during Countdown and Playing; free otherwise. `#[derive(Resource, Default)] pub struct CursorFree(pub bool)`: when true, cursor is free and look/fire are disabled.
5. `pub struct Shot` event (message/event per pinned Bevy), sent on left mouse `just_pressed` in Playing when CursorFree is false.
6. Register in CorePlugin.
7. Done when: check + clippy clean. Then user tests: F2, look around, sens/FOV changes in Settings apply next round.

Scope: Target Files + Progress.md. Do not read or modify files outside of this scope.