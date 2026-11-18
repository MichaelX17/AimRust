Target Files: src/core/round.rs, src/core/ui/task_select.rs, src/core/ui/time_select.rs, src/core/ui/pause.rs, src/core/ui/results.rs, src/core/ui/mod.rs, src/core/mod.rs, src/core/debug.rs
Read-Only Files: none
Task_11: Round flow + pause

1. DELETE src/core/debug.rs and its registration (all TEMP-DEBUG code).
2. task_select.rs: egui list of TaskRegistry (name + description). Empty -> "No tasks installed". Click -> set SelectedTask, go TimeSelect. Back -> Menu.
3. time_select.rs: preset buttons 0:30, 1:00, 2:00, 3:00, 5:00 -> set `RoundDuration(pub u32)` secs, go Countdown. Back -> TaskSelect.
4. round.rs: `RoundTimer { duration: f32, remaining: f32 }` resource set on OnEnter(Countdown). Countdown 3-2-1 (1 s each, big centered egui text) -> Playing. Timer ticks only in Playing; at 0 -> Results.
5. pause.rs: Esc in Playing -> Paused (timer and task systems stop, CursorFree true or cursor released). Egui: Resume (-> Playing), Quit (-> Menu, discard, save nothing). Esc in Paused = Resume.
6. results.rs: placeholder "Round over" + Back to Menu button.
7. Ensure Paused -> Playing does NOT respawn arena/task/crosshair (lifecycle in Main.md).
8. Done when: check + clippy clean. Then user tests: full flow Menu > Sandbox > 30 s > countdown > round > Results > Menu; pause/resume/quit.

Scope: Target Files + Progress.md. Do not read or modify files outside of this scope.