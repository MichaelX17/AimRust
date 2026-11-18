Target Files: src/core/score.rs, src/core/hud.rs, src/core/ui/results.rs, src/core/ui/mod.rs, src/core/mod.rs
Read-Only Files: none
Task_12: Scoring + HUD + results

1. score.rs: `RoundScore { points: i32, hits: u32, misses: u32, tracking_ms: f64 }` resource (pub fields), reset on OnEnter(Countdown).
2. Mode = TaskRegistry entry of SelectedTask `.scoring`.
   - Click: TargetHit -> +10, hits+1. Miss and TargetExpired -> -3, misses+1. Never show score below 0 (points may go negative internally; display max(0)).
   - Tracking: each frame in Playing with AimingAt Some -> tracking_ms += delta_secs * 1000 (f64, exact); points = floor(tracking_ms). Shot/Miss ignored.
3. hud.rs: native UI, spawned OnEnter(Countdown), despawned OnEnter(Results)/OnEnter(Menu): time left mm:ss (top-left), score (top-center), accuracy % = hits/(hits+misses) for Click or on-target ms for Tracking (top-right).
4. results.rs: replace placeholder: score, hits, misses, accuracy (Click) or time on target ms + % of round (Tracking). Back to Menu button. (Best score comes in Task_13.)
5. Done when: check + clippy clean. Then user tests: Sandbox round scores +10/-3 correctly, expired targets count as misses.

Scope: Target Files + Progress.md. Do not read or modify files outside of this scope.