mod components;
mod systems;
mod resources;

use bevy::prelude::*;
use bevy_screen_diagnostics::{ScreenDiagnosticsPlugin, ScreenFrameDiagnosticsPlugin};

fn main() {
    println!("Hello, world!");
    App::new()
        .add_plugins(DefaultPlugins.set(
            WindowPlugin {
                primary_window: Some(Window {
                    fit_canvas_to_parent: true,
                    canvas: Some("#bevy".to_owned()),
                    ..default()
                }),
                ..default()
            }
        ))
        .add_plugin(ScreenDiagnosticsPlugin::default())
        .add_plugin(ScreenFrameDiagnosticsPlugin)
        .add_startup_system(systems::setup)
        .add_startup_system(systems::spawn_player)
        .add_systems(
            (
                systems::remove_dead,
                systems::spawn_enemies,
                systems::move_enemies,
                systems::move_player,
                systems::whip_enemies,
                systems::camera_look_at,
            ).chain()
        )
        .run();
}