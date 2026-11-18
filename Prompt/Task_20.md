Target Files: src/tasks/tracking_shoot/mod.rs, src/tasks/mod.rs
Read-Only Files: docs/ADDING_A_TASK.md
Task_20: Tracking Shoot (Click)

Follow docs/ADDING_A_TASK.md only. INFO { id "tracking_shoot", name "Tracking: Shoot", scoring Click }. Add 1 line to register_tasks!.
- One sphere (radius 0.3) moving on the plane z = -15 inside x -5..5, y 0.8..4, speed 4 m/s, same smooth random-direction motion as Tracking Follow (reimplement locally, do not import it).
- Lifetime 4 s. On TargetHit: despawn, spawn a new one at a random position with a new direction. On TargetExpired (counts as a miss in core): despawn and respawn the same way.
- Marker `TrackingShootEntity`; cleanup per lifecycle.
- Done when: check + clippy clean. User tests a 30 s round.

Scope: Target Files + Read-Only Files + Progress.md. Do not read or modify files outside of this scope.