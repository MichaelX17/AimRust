Target Files: src/core/crosshair.rs, src/core/mod.rs
Read-Only Files: none
Task_08: Crosshair

1. crosshair.rs: native Bevy UI nodes, exactly screen-centered, pixel-snapped.
   - Dot: square of `thickness` px.
   - Cross: 4 bars, length `size`, width `thickness`, center `gap`.
   - Circle: ring, diameter `size`, border width `thickness`, 50% radius.
   - T: cross without the top bar.
   - Outline: 1 px black border around every part.
2. Color from Settings.crosshair.color.
3. Spawn on OnEnter(Countdown), despawn on OnEnter(Results) and OnEnter(Menu). Rebuild when Settings.crosshair changes.
4. Register in CorePlugin.
5. Done when: check + clippy clean. Then user tests: F2, crosshair visible and centered, all 4 styles correct.

Scope: Target Files + Progress.md. Do not read or modify files outside of this scope.