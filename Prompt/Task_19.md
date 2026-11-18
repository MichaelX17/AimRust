Target Files: src/tasks/tracking_follow/mod.rs, src/tasks/mod.rs
Read-Only Files: docs/ADDING_A_TASK.md
Task_19: Tracking Follow (Tracking)

Follow docs/ADDING_A_TASK.md only. INFO { id "tracking_follow", name "Tracking: Follow", scoring Tracking }. Add 1 line to register_tasks!.
- One sphere (radius 0.35) moving on the plane z = -15 inside x -5..5, y 0.8..4.
- Speed 3 m/s. Every 0.5-1.5 s (random) pick a new random direction; smooth the velocity toward it (lerp) so motion is fluid, never teleporting. Bounce at bounds.
- No firing logic, no lifetime. Core scores time on target (1 point per ms). Marker `TrackingFollowEntity`; cleanup per lifecycle.
- Done when: check + clippy clean. User tests a 30 s round; score ~ ms on target.

Scope: Target Files + Read-Only Files + Progress.md. Do not read or modify files outside of this scope.