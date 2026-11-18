Target Files: src/tasks/sixshot/mod.rs, src/tasks/mod.rs
Read-Only Files: docs/ADDING_A_TASK.md
Task_17: Sixshot (Click)

Follow docs/ADDING_A_TASK.md only. INFO { id "sixshot", name "Sixshot", scoring Click }. Add 1 line to register_tasks!.
- 6 sphere targets (radius 0.2) visible at all times on the plane z = -15, random positions inside x -4..4, y 0.8..3.8, minimum 1.0 m between targets.
- On TargetHit: despawn it, spawn a replacement at a new random valid position.
- No lifetime. Marker `SixshotEntity`; cleanup per lifecycle.
- Done when: check + clippy clean. User tests a 30 s round.

Scope: Target Files + Read-Only Files + Progress.md. Do not read or modify files outside of this scope.