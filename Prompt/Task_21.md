Target Files: src/tasks/flick/mod.rs, src/tasks/mod.rs
Read-Only Files: docs/ADDING_A_TASK.md
Task_21: Flick Reaction (Click)

Follow docs/ADDING_A_TASK.md only. INFO { id "flick", name "Flick / Reaction", scoring Click }. Add 1 line to register_tasks!.
- One sphere (radius 0.25) at a time on the plane z = -15, random position inside x -6..6, y 0.6..4.2.
- Spawn after a random delay 0.3-1.0 s from the previous target's removal. Lifetime 1.2 s.
- On TargetHit: despawn, start the next delay. On TargetExpired (miss in core): despawn, start the next delay.
- Marker `FlickEntity`; cleanup per lifecycle.
- Done when: check + clippy clean. User tests a 30 s round.

Scope: Target Files + Read-Only Files + Progress.md. Do not read or modify files outside of this scope.