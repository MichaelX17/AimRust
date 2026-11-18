Target Files: src/core/hit.rs, src/props/targets.rs, src/props/mod.rs, src/core/mod.rs
Read-Only Files: none (Shot, PlayerCamera, Settings are in Progress.md Interfaces)
Task_09: Targets + hitscan

1. hit.rs:
   - `#[derive(Component)] struct Target { shape: TargetShape, lifetime: Option<f32> }`
   - `enum TargetShape { Sphere { radius: f32 }, Box { half_extents: Vec3 } }` (box axis-aligned, no rotation)
   - events: `TargetHit { entity: Entity }`, `Miss`, `TargetExpired { entity: Entity }`
   - `#[derive(Resource, Default)] struct AimingAt(pub Option<Entity>)`
2. Each frame in Playing: ray from PlayerCamera center along its forward. Analytic ray-sphere and ray-AABB against each Target (center = GlobalTransform translation, scale ignored). Nearest hit -> AimingAt.
3. On `Shot`: AimingAt Some -> send TargetHit, else send Miss.
4. Lifetime: tick down in Playing only; at <= 0 send TargetExpired ONCE per target (marker component `Expired`). Do not despawn; the task decides.
5. targets.rs: `TargetMaterial` resource (one shared material, color = Settings.colors.target, updated on change). Builders: `spawn_sphere(commands, meshes, &TargetMaterial, pos: Vec3, radius: f32, lifetime: Option<f32>) -> Entity` and `spawn_box(..., half_extents: Vec3, ...)`. Both add Target.
6. Register in CorePlugin / PropsPlugin.
7. Done when: check + clippy clean.

Scope: Target Files + Progress.md. Do not read or modify files outside of this scope.