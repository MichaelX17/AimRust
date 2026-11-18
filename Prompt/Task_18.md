Target Files: src/tasks/spidershot/mod.rs, src/tasks/mod.rs
Read-Only Files: docs/ADDING_A_TASK.md
Task_18: Spidershot (Click)

Follow docs/ADDING_A_TASK.md only. INFO { id "spidershot", name "Spidershot", scoring Click }. Add 1 line to register_tasks!.
- One sphere target (radius 0.25) at a time on the plane z = -15, relative to center (0, 1.7).
- Alternate: odd target spawns near center (offset radius <= 1.0 m), even target spawns far out (offset radius 3.0-5.0 m, random angle, clamped inside x -6..6, y 0.5..4.5).
- On TargetHit: despawn and spawn the next one. No lifetime. Marker `SpidershotEntity`; cleanup per lifecycle.
- Done when: check + clippy clean. User tests a 30 s round.

Scope: Target Files + Read-Only Files + Progress.md. Do not read or modify files outside of this scope.