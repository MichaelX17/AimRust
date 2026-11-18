# Adding a task

Self-contained guide. Use it with `src/tasks/_template/mod.rs`; no other docs needed.

## Folder layout and registration

```
src/tasks/<name>/mod.rs   ->  pub const INFO: TaskInfo;  pub fn plugin() -> impl Plugin;
```

Add one line in `src/tasks/mod.rs` (the only shared file you touch):

```rust
register_tasks!(sandbox, <name>);
```

That macro declares the modules, registers every `plugin()` and pushes every `INFO`
into `TaskRegistry`. The app must compile with zero tasks (`register_tasks!();`).
`_template` is compiled but never registered; copy it, don't reference it.

Remove a task: delete `src/tasks/<name>/` and its name from `register_tasks!`.

## INFO and ScoringMode

```rust
pub const INFO: TaskInfo = TaskInfo {
    id: "<unique id>",            // &'static str, used in scores.toml and SelectedTask
    name: "<display name>",
    description: "<one line>",
    scoring: ScoringMode::Click,  // or ScoringMode::Tracking
};
```

- `Click`: core scores +10 per `TargetHit`, -3 per `Miss` and per `TargetExpired`.
  Accuracy = hits / (hits + misses). Displayed score is clamped at 0.
- `Tracking`: core adds `delta * 1000` to `tracking_ms` every frame while `AimingAt`
  is `Some`; score = floor(ms). Shots/misses are ignored.

Tasks never compute or display score, and never touch `RoundScore`, HUD or storage.

## Core API (exact signatures)

Bevy 0.20 uses messages: read them with `MessageReader<T>`.

```rust
use crate::core::config::Settings;                       // Resource, read-only
use crate::core::hit::{AimingAt, Target, TargetExpired, TargetHit, Miss};
use crate::core::task_api::{task_playing, task_selected, ScoringMode, TaskInfo};
use crate::props::targets::{spawn_box, spawn_sphere, TargetMaterial};

// Resources (read): Settings, TargetMaterial, AimingAt(pub Option<Entity>)
// Components: Target { pub shape: TargetShape, pub lifetime: Option<f32> }
//             TargetShape::Sphere { radius: f32 } | TargetShape::Box { half_extents: Vec3 }
// Messages:   TargetHit { pub entity: Entity }, Miss, TargetExpired { pub entity: Entity }

pub fn spawn_sphere(
    commands: &mut Commands, meshes: &mut Assets<Mesh>, material: &TargetMaterial,
    pos: Vec3, radius: f32, lifetime: Option<f32>,
) -> Entity;

pub fn spawn_box(
    commands: &mut Commands, meshes: &mut Assets<Mesh>, material: &TargetMaterial,
    pos: Vec3, half_extents: Vec3, lifetime: Option<f32>,
) -> Entity;

// Run conditions
pub fn task_selected(id: &'static str) -> /* true when SelectedTask == id */;
pub fn task_playing(id: &'static str) -> /* true when Playing && SelectedTask == id */;
```

`TargetHit` fires only for clicks whose crosshair ray hits a target (`AimingAt`);
`Miss` fires for clicks that hit nothing; `TargetExpired` fires once per target when
a finite `lifetime` reaches 0 (the core does not despawn expired targets — you decide:
reposition and reset `target.lifetime`, or despawn it).

## Round scene lifecycle

- Setup: `OnEnter(AppState::Countdown)` with `.run_if(task_selected(INFO.id))`.
- Gameplay: `Update` systems with `.run_if(task_playing(INFO.id))`.
- Cleanup: `OnEnter(AppState::Results)` **and** `OnEnter(AppState::Menu)`
  (Quit from pause goes straight to Menu). Despawn everything you spawned, using your
  own marker component.
- Paused/Playing transitions do not re-enter Countdown, so never spawn on `OnEnter(Playing)`.
- `OnEnter(Results)` means the round finished normally; Quit never reaches Results.

## Arena

Room interior: x -10..10, y 0..6, z -30..5. Player is fixed at (0, 1.7, 0) facing -Z.
Targets typically live at z -6..-25, x -8..8, y 0.8..5.

## Spawn rules

- A target must never respawn on the spawn point it just vacated. When it is hit
  (or expires), exclude that point from the candidate list and pick the new point
  among the others; the old point may only be reused after a spawn happened
  somewhere else.

```rust
let vacated = cell_of(hit.entity);
let free: Vec<usize> = (0..CELLS)
    .filter(|cell| !occupied[*cell] && *cell != vacated)
    .collect();
```

- Apply this in every level (gridshot, sixshot, spidershot, ...), both on hit and on
  expiry respawns. With continuous positions, respawn at least ~1 m away from the
  point that was just vacated.

## Rules

- Never read or write config files; `Settings` is read-only.
- Never draw or handle menus, HUD, pause, timer or results.
- Never score; emit/consume only the messages above.
- Never build meshes/materials yourself; use the props builders (sphere/box) and
  `TargetMaterial`.
- No new crates, no changes to core, no renames outside your folder.
- If you need anything outside this contract: stop and report it.

## Worked example (~40 lines)

```rust
use bevy::prelude::*;

use crate::core::hit::TargetHit;
use crate::core::state::AppState;
use crate::core::task_api::{ScoringMode, TaskInfo, task_playing, task_selected};
use crate::props::targets::{TargetMaterial, spawn_sphere};

pub const INFO: TaskInfo = TaskInfo {
    id: "one_target",
    name: "One Target",
    description: "One sphere, flips side on hit",
    scoring: ScoringMode::Click,
};

#[derive(Component)]
struct MyTarget;

pub fn plugin() -> impl Plugin {
    MyTaskPlugin
}

struct MyTaskPlugin;

impl Plugin for MyTaskPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
                OnEnter(AppState::Countdown),
                setup.run_if(task_selected(INFO.id)),
            )
            .add_systems(Update, move_on_hit.run_if(task_playing(INFO.id)))
            .add_systems(OnEnter(AppState::Results), cleanup)
            .add_systems(OnEnter(AppState::Menu), cleanup);
    }
}

fn setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, material: Res<TargetMaterial>) {
    let entity = spawn_sphere(
        &mut commands, &mut meshes, &material,
        Vec3::new(0.0, 1.7, -10.0), 0.3, None,
    );
    commands.entity(entity).insert(MyTarget);
}

fn move_on_hit(mut hits: MessageReader<TargetHit>, mut targets: Query<&mut Transform, With<MyTarget>>) {
    for hit in hits.read() {
        if let Ok(mut transform) = targets.get_mut(hit.entity) {
            transform.translation.x = -transform.translation.x;
        }
    }
}

fn cleanup(mut commands: Commands, targets: Query<Entity, With<MyTarget>>) {
    for entity in &targets {
        commands.entity(entity).despawn();
    }
}
```

Tracking task: same skeleton, `ScoringMode::Tracking`, and a `task_playing` system that
moves targets; core does the +ms scoring via `AimingAt`.

## Checklist

- [ ] folder `src/tasks/<name>/mod.rs` with `INFO` (unique id) and `plugin()`
- [ ] added `<name>` to `register_tasks!(...)`
- [ ] setup gated by `task_selected(INFO.id)` on `OnEnter(Countdown)`
- [ ] gameplay gated by `task_playing(INFO.id)`
- [ ] cleanup on both `OnEnter(Results)` and `OnEnter(Menu)`
- [ ] all spawned entities carry your marker and are despawned
- [ ] targets spawned only through props builders with `TargetMaterial`
- [ ] respawn never reuses the spawn point a target just vacated
- [ ] no config/menu/HUD/scoring/file code, no new crates
- [ ] `cargo check --message-format short` and `cargo clippy --message-format short` clean
- [ ] play a full round from the menu and check no leftovers after Quit and after Results

## Common mistakes

- Missing `task_selected`/`task_playing` -> your task spawns during other tasks.
- Cleanup only on Results -> entities survive Quit to Menu.
- Despawning a target on `TargetExpired` without considering core scoring: it still
  counts as a miss once; that is intended.
- Spawning on `OnEnter(Playing)` -> duplicate scene after every pause/resume.
- Using `EventReader`/`EventWriter` -> Bevy 0.20 uses `MessageReader`/`MessageWriter`.
- Building `Mesh3d`/`StandardMaterial` yourself instead of the props builders.
- Respawning on the cell that was just hit: always exclude that cell (see Spawn rules).
- Duplicate `INFO.id` (breaks per-task scores and `SelectedTask`).
- Forgetting the `register_tasks!` line: the folder is ignored and never compiled.
