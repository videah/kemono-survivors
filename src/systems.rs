use bevy::prelude::*;
use bevy::render::camera::ScalingMode;
use rand::Rng;

use crate::components::*;

pub fn setup(mut commands: Commands) {
    let mut camera_bundle = Camera2dBundle::default();
    camera_bundle.projection.scaling_mode = ScalingMode::FixedVertical(512.0);
    commands.spawn((
        MainCamera,
        camera_bundle
    ));
}

pub fn spawn_player(mut commands: Commands) {
    commands.spawn((
        Player,
        LookAt,
        SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.8, 0.7, 0.6),
                custom_size: Some(Vec2::new(32.0, 32.0)),
                ..Default::default()
            },
            ..Default::default()
        }
    ));
}

pub fn move_player(
    time: Res<Time>,
    keys: Res<Input<KeyCode>>,
    mut player_query: Query<&mut Transform, With<Player>>,
) {
    let mut direction = Vec2::ZERO;
    if keys.any_pressed([KeyCode::Up, KeyCode::W]) {
        direction.y += 1.;
    }
    if keys.any_pressed([KeyCode::Down, KeyCode::S]) {
        direction.y -= 1.;
    }
    if keys.any_pressed([KeyCode::Right, KeyCode::D]) {
        direction.x += 1.;
    }
    if keys.any_pressed([KeyCode::Left, KeyCode::A]) {
        direction.x -= 1.;
    }
    if direction == Vec2::ZERO {
        return;
    }

    let move_speed = 200.0;
    let move_delta = (direction * move_speed).extend(0.);

    for mut transform in player_query.iter_mut() {
        transform.translation += (move_delta) * time.delta_seconds();
    }
}

#[allow(clippy::type_complexity)]
pub fn camera_look_at(
    mut set: ParamSet<(
        Query<(&mut Transform, With<MainCamera>)>,
        Query<(&mut Transform, With<LookAt>)>,
    )>
) {
    let look_at_post = {
        match set.p1().get_single() {
            Ok(look_at) => look_at.0.translation,
            Err(e) => {
                info!("No look at entity found: {:?}", e);
                return;
            }
        }
    };

    match set.p0().get_single_mut() {
        Ok(mut camera) => {
            camera.0.translation.x = look_at_post.x;
            camera.0.translation.y = look_at_post.y;
        }
        Err(e) => {
            info!("No camera found: {:?}", e);
        }
    }
}

/// Spawn enemies over time at a random position outside of the screen.
pub fn spawn_enemies(
    mut commands: Commands,
    time: Res<Time>,
) {
    if time.elapsed_seconds_f64() % 1.0 > 0.1 {
        return;
    }

    let mut rng = rand::thread_rng();
    let x = rng.gen_range(-1000.0..1000.0);
    let y = rng.gen_range(-1000.0..1000.0);

    // Random color
    let r = rng.gen_range(0.0..1.0);
    let g = rng.gen_range(0.0..1.0);
    let b = rng.gen_range(0.0..1.0);
    let color = Color::rgb(r, g, b);

    commands.spawn((
        Enemy,
        SpriteBundle {
            sprite: Sprite {
                color,
                custom_size: Some(Vec2::new(32.0, 32.0)),
                ..Default::default()
            },
            transform: Transform {
                translation: Vec3::new(x, y, 0.0),
                ..Default::default()
            },
            ..Default::default()
        }
    ));
}

#[allow(clippy::type_complexity)]
pub fn move_enemies(
    time: Res<Time>,
    mut set: ParamSet<(
        Query<(&mut Transform, With<Enemy>)>,
        Query<(&mut Transform, With<Player>)>,
    )>,
) {
    let player_pos = {
        match set.p1().get_single() {
            Ok(player) => player.0.translation,
            Err(e) => {
                info!("No player found: {:?}", e);
                return;
            }
        }
    };

    // Move enemies towards the player
    for mut enemy in set.p0().iter_mut() {
        let enemy_pos = enemy.0.translation;
        let direction = (player_pos - enemy_pos).normalize();
        let move_speed = 50.0;
        let move_delta = direction * move_speed;
        enemy.0.translation += move_delta * time.delta_seconds();
    }

}