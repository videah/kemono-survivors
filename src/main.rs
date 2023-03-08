mod components;
mod systems;

use bevy::prelude::*;
use bevy_screen_diagnostics::{ScreenDiagnosticsPlugin, ScreenFrameDiagnosticsPlugin};

fn main() {
    println!("Hello, world!");
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(ScreenDiagnosticsPlugin::default())
        .add_plugin(ScreenFrameDiagnosticsPlugin)
        .add_startup_system(systems::setup)
        .add_startup_system(systems::spawn_player)
        .add_system(systems::move_player)
        .add_system(systems::camera_look_at.after(systems::move_player))
        .run();
}