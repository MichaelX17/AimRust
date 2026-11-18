Target Files: src/tasks/gridshot/mod.rs, src/tasks/mod.rs
Read-Only Files: docs/ADDING_A_TASK.md
Task_16: Gridshot (Click)

Follow docs/ADDING_A_TASK.md only. INFO { id "gridshot", name "Gridshot", scoring Click }. Add 1 line to register_tasks!.
- 3x3 grid of cells on the plane z = -12, spacing 1.2 m, centered at x 0, y 1.7.
- Always exactly 3 sphere targets (radius 0.3) on random distinct cells.
- On TargetHit: despawn that target, spawn a new one on a random free cell (different from all current).
- No lifetime. Marker `GridshotEntity`; cleanup per lifecycle.
- Done when: check + clippy clean. User tests a 30 s round.

Scope: Target Files + Read-Only Files + Progress.md. Do not read or modify files outside of this scope.