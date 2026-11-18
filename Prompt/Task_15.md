Target Files: docs/ADDING_A_TASK.md
Read-Only Files: src/tasks/_template/mod.rs, src/tasks/mod.rs, src/tasks/sandbox/mod.rs
Task_15: ADDING_A_TASK.md + template check

1. Write docs/ADDING_A_TASK.md: self-contained, terse guide so any AI (DeepSeek, Claude, Gemini) can add a task using only this file + `_template`. Sections:
   - Folder layout and the 1-line registration in register_tasks!
   - INFO fields and ScoringMode (Click vs Tracking) and what each implies
   - Core contract with exact signatures: Settings (read-only), Target + TargetShape, props spawn_sphere/spawn_box, events TargetHit/Miss/TargetExpired, AimingAt, task_selected/task_playing, round scene lifecycle (setup OnEnter(Countdown), cleanup OnEnter(Results)+OnEnter(Menu))
   - Arena coordinates: room x -10..10, y 0..6, z -30..5, player at (0,1.7,0) facing -Z
   - Rules: tasks never read/write config, never draw menus, never score, never build meshes outside props
   - How to remove a task
   - Worked example (minimal click task, ~40 lines)
   - Checklist + common mistakes
2. Verify the template: temporarily copy `_template` to `_verify`, add to register_tasks!, run check (must pass), then delete `_verify` and the line, run check again (must pass). Do not run the game.
3. Done when: guide written, both checks clean, `_verify` gone.

Scope: Target Files + Read-Only Files + Progress.md. Do not read or modify files outside of this scope.