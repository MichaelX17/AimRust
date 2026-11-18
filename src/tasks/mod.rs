use bevy::prelude::*;

use crate::core::task_api::TaskRegistry;

/// Registers task modules: `mod a; mod b;` plus a `TasksPlugin` that adds every
/// `a::plugin()` and pushes every `a::INFO` into [`TaskRegistry`].
///
/// Add a task: `register_tasks!(gridshot, sixshot);`
/// Remove a task: delete its folder and its line here.
macro_rules! register_tasks {
    ($($task:ident),* $(,)?) => {
        $(mod $task;)*

        #[allow(dead_code)]
        mod _template;

        pub struct TasksPlugin;

        impl Plugin for TasksPlugin {
            fn build(&self, app: &mut App) {
                #[allow(unused_mut)]
                let mut registry = TaskRegistry::default();
                $(
                    app.add_plugins($task::plugin());
                    registry.0.push($task::INFO);
                )*
                app.insert_resource(registry);
            }
        }
    };
}

register_tasks!(sandbox, gridshot, sixshot, spidershot, tracking_follow, tracking_shoot, flick);
