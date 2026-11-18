Target Files: src/props/environment.rs, src/core/arena.rs, src/core/debug.rs, src/props/mod.rs, src/core/mod.rs
Read-Only Files: none
Task_06: Arena + props environment + temp debug

1. environment.rs: `RoomMaterials` resource (one shared StandardMaterial each for floor, ceiling, walls) created at Startup from Settings.colors; system updating their colors when Settings changes. `pub fn spawn_room(commands, meshes, &RoomMaterials)` spawns floor, ceiling, 4 walls (primitives, no textures). Interior: x -10..10, y 0..6, z -30..5. Each entity gets marker `ArenaEntity`.
2. arena.rs: OnEnter(Countdown): spawn_room + ONE DirectionalLight (shadows off) + AmbientLight (marked ArenaEntity). Cleanup (despawn all ArenaEntity) on OnEnter(Results) and OnEnter(Menu). Add `ArenaEntity` marker as pub.
3. debug.rs (TEMPORARY, header comment `// TEMP-DEBUG: delete in Task_11`): F2 in Menu -> Countdown; OnEnter(Countdown) -> immediately set Playing; Esc in Playing -> Menu.
4. Done when: check + clippy clean. Then user tests: `cargo run`, press F2 in menu: room visible; Esc returns to menu and room disappears. Do not run the game yourself.

Scope: Target Files + Progress.md. Do not read or modify files outside of this scope.