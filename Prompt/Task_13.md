Target Files: src/core/storage.rs, src/core/ui/results.rs, src/core/mod.rs
Read-Only Files: none
Task_13: Storage scores.toml

1. storage.rs: `Scores` resource (serde): per task id -> { recent: Vec<RoundRecord> (max 15), best: map duration_secs (as string key) -> BestRecord { score, unix_secs } }. `RoundRecord { unix_secs: u64, duration: u32, score: i32, hits: u32, misses: u32, tracking_ms: u64 }`. Use std::time for timestamps (no chrono).
2. File: dirs::config_dir()/aim_trainer/scores.toml. Startup load; missing -> empty; parse error -> rename to scores.toml.bak + empty. No unwrap on I/O.
3. OnEnter(Results) (round finished normally only; Quit never reaches Results): append record, trim to last 15, update best if score is strictly higher, write file once. Set `LastOutcome { new_best: bool, best: i32 }` resource.
4. results.rs: show best for this task + duration, NEW BEST badge when applicable, and a scroll list of the last 15 rounds for this task (score, hits, misses, duration).
5. Done when: check + clippy clean. Then user tests: play 2 rounds; scores.toml updated; best never lost; NEW BEST shows.

Scope: Target Files + Progress.md. Do not read or modify files outside of this scope.