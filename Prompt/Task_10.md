Target Files: src/tasks/sandbox/mod.rs, src/tasks/mod.rs
Read-Only Files: none (all APIs are in Progress.md Interfaces)
Task_10: Sandbox task

1. sandbox/mod.rs: INFO { id "sandbox", name "Sandbox", description "Test level for settings and hit detection", scoring Click }. Add `sandbox` to register_tasks!.
2. Setup (OnEnter(Countdown), task_selected): 3 static spheres (radius 0.15 / 0.3 / 0.5) at z -8 / -14 / -20; 2 static boxes (half 0.3) at z -10 / -18; 1 sphere oscillating on x +-4 at z -12; 1 sphere with lifetime 3 s at z -15. All via props builders. All marked `SandboxEntity`.
3. Gameplay (task_playing): oscillation; on TargetHit move that target to a random position inside x -6..6, y 0.8..4, same z; on TargetExpired reset its lifetime and reposition.
4. Debug overlay (egui, top-left): FPS (bevy diagnostics), sens, fov_h, last event name (Hit/Miss/Expired).
5. F1 toggles an egui window with `settings_panel`; while open set CursorFree(true), else false.
6. Cleanup on OnEnter(Results) and OnEnter(Menu): despawn SandboxEntity.
7. Done when: check + clippy clean. Then user tests: F2 (debug) > hits/misses logged, expiring target works, F1 panel changes colors/crosshair/sens live.

Scope: Target Files + Progress.md. Do not read or modify files outside of this scope.