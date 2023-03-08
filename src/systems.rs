use bevy::prelude::*;
use bevy::render::camera::ScalingMode;

use crate::components::*;

pub fn setup(mut commands: Commands) {
    let mut camera_bundle = Camera2dBundle::default();
    camera_bundle.projection.scaling_mode = ScalingMode::FixedVertical(128.0);
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
    mut player_query: Query<&mut Transform, With<Player>>
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
            },
        }
    };

    match set.p0().get_single_mut() {
        Ok(mut camera) => {
            camera.0.translation.x = look_at_post.x;
            camera.0.translation.y = look_at_post.y;
        },
        Err(e) => {
            info!("No camera found: {:?}", e);
        },
    }
}