use bevy::prelude::*;
use bevy::render::camera::ScalingMode;
use rand::distributions::{WeightedIndex, Distribution};
use rand::Rng;

use crate::components::*;
use crate::resources::{EnemySpawnConfig, WeaponConfig};

pub fn setup(mut commands: Commands) {
    // Camera
    let mut camera_bundle = Camera2dBundle::default();
    camera_bundle.projection.scaling_mode = ScalingMode::FixedVertical(512.0);
    commands.spawn((
        MainCamera,
        camera_bundle
    ));

    // Enemy spawner
    commands.insert_resource(EnemySpawnConfig {
        timer: Timer::from_seconds(0.2, TimerMode::Repeating),
    });

    // Weapon timers
    commands.insert_resource(WeaponConfig {
        whip_timer: Timer::from_seconds(2.0, TimerMode::Repeating),
    });
}

pub fn spawn_player(mut commands: Commands) {
    commands.spawn((
        Player,
        LookAt,
        Health(100.0),
        MovementDirection(Vec2::ZERO),
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
    mut player_query: Query<(&mut Transform, &mut MovementDirection, With<Player>)>,
) {
    // Get player transform and movement direction
    let (mut transform, mut direction, _) = player_query.single_mut();
    direction.0 = Vec2::ZERO;

    if keys.any_pressed([KeyCode::Up, KeyCode::W]) {
        direction.0.y += 1.0;
    }
    if keys.any_pressed([KeyCode::Down, KeyCode::S]) {
        direction.0.y -= 1.0;
    }
    if keys.any_pressed([KeyCode::Right, KeyCode::D]) {
        direction.0.x += 1.0;
    }
    if keys.any_pressed([KeyCode::Left, KeyCode::A]) {
        direction.0.x -= 1.0;
    }
    if direction.0 == Vec2::ZERO {
        return;
    }

    let move_speed = 200.0;
    let move_delta = (direction.0 * move_speed).extend(0.);

    transform.translation += (move_delta) * time.delta_seconds();
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
    mut windows: Query<&mut Window>,
    mut config: ResMut<EnemySpawnConfig>,
    player_pos: Query<&Transform, With<Player>>,
    player_direction: Query<&MovementDirection, With<Player>>,
) {
    config.timer.tick(time.delta());

    if config.timer.finished() {
        // Get window size
        let window = windows.single_mut();
        let window_size = Vec2::new(window.width(), window.height());

        // Get player position
        let player_pos = player_pos.single();

        // Define the viewport boundary position using the player, who is the center of the screen.
        let viewport_boundary = Vec2::new(
            player_pos.translation.x - window_size.x / 2.0,
            player_pos.translation.y - window_size.y / 2.0,
        );

        enum SpawnSide {
            Top,
            Bottom,
            Left,
            Right,
        }

        // Get a random side to spawn the enemy on, weight towards the direction the player is moving.
        // Adjust weights based on player direction
        let player_direction = player_direction.single().0;
        println!("Player direction: {:?}", player_direction);
        let mut weights = [1.0; 4];

        // Adjust weights based on player direction
        const WEIGHT_FACTOR: f32 = 2.0;
        match player_direction {
            dir if dir.x < 0.0 => {
                weights[2] += WEIGHT_FACTOR;
                weights[3] -= WEIGHT_FACTOR;
            },
            dir if dir.x > 0.0 => {
                weights[2] -= WEIGHT_FACTOR;
                weights[3] += WEIGHT_FACTOR;
            },
            dir if dir.y > 0.0 => {
                weights[0] += WEIGHT_FACTOR;
                weights[1] -= WEIGHT_FACTOR;
            },
            dir if dir.y < 0.0 => {
                weights[0] -= WEIGHT_FACTOR;
                weights[1] += WEIGHT_FACTOR;
            },
            _ => {}
        }

        // Make sure weights are not negative
        weights.iter_mut().for_each(|weight| {
            *weight = weight.max(0.0);
        });

        println!("Weights: {:?}", weights);

        let mut rng = rand::thread_rng();
        let spawn_side = match WeightedIndex::new(weights).unwrap().sample(&mut rng) {
            0 => SpawnSide::Top,
            1 => SpawnSide::Bottom,
            2 => SpawnSide::Left,
            3 => SpawnSide::Right,
            _ => unreachable!(),
        };

        // Get a random position on the side
        let spawn_pos = match spawn_side {
            SpawnSide::Top => {
                let x = rng.gen_range(
                    viewport_boundary.x..viewport_boundary.x + window_size.x,
                );
                let y = viewport_boundary.y + window_size.y;
                Vec2::new(x, y)
            }
            SpawnSide::Bottom => {
                let x = rng.gen_range(
                    viewport_boundary.x..viewport_boundary.x + window_size.x,
                );
                let y = viewport_boundary.y;
                Vec2::new(x, y)
            }
            SpawnSide::Left => {
                let x = viewport_boundary.x;
                let y = rng.gen_range(
                    viewport_boundary.y..viewport_boundary.y + window_size.y,
                );
                Vec2::new(x, y)
            }
            SpawnSide::Right => {
                let x = viewport_boundary.x + window_size.x;
                let y = rng.gen_range(
                    viewport_boundary.y..viewport_boundary.y + window_size.y,
                );
                Vec2::new(x, y)
            }
        };

        // Random color
        let r = rng.gen_range(0.0..1.0);
        let g = rng.gen_range(0.0..1.0);
        let b = rng.gen_range(0.0..1.0);
        let color = Color::rgb(r, g, b);

        commands.spawn((
            Enemy,
            Health(10.0),
            SpriteBundle {
                sprite: Sprite {
                    color,
                    custom_size: Some(Vec2::new(32.0, 32.0)),
                    ..Default::default()
                },
                transform: Transform {
                    translation: spawn_pos.extend(0.0),
                    ..Default::default()
                },
                ..Default::default()
            }
        ));
    }
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

/// Whip enemies that are close to the player every 2 seconds.
#[allow(clippy::type_complexity)]
pub fn whip_enemies(
    time: Res<Time>,
    mut config: ResMut<WeaponConfig>,
    mut set: ParamSet<(
        Query<(&mut Transform, &mut Health, With<Enemy>)>,
        Query<(&mut Transform, With<Player>)>,
    )>,
) {
    config.whip_timer.tick(time.delta());

    if config.whip_timer.finished() {
        let player_pos = match set.p1().get_single() {
            Ok(player) => player.0.translation,
            Err(e) => {
                info!("No player found: {:?}", e);
                return;
            }
        };

        for (mut enemy, mut health, _) in set.p0().iter_mut() {

            let enemy_pos = enemy.translation;
            let distance = (enemy_pos - player_pos).length();
            if distance < 100.0 {
                // Whip the enemy, knocking it back a bit based on its distance and angle from the player.
                let direction = (enemy_pos - player_pos).normalize();
                let angle = direction.y.atan2(direction.x);
                let knockback_distance = (100.0 - distance) / 100.0 * 50.0; // Adjust the 50.0 value as needed.
                let knockback_vector = Vec2::new(angle.cos(), angle.sin()) * knockback_distance;
                enemy.translation += Vec3::new(knockback_vector.x, knockback_vector.y, 0.0);

                // Damage the enemy
                health.0 -= 5.0;
            }
        }
    }
}

/// Remove entities that have 0 health.
pub fn remove_dead(
    mut commands: Commands,
    mut query: Query<(Entity, &Health), Without<Player>>,
) {
    for (entity, health) in query.iter_mut() {
        if health.0 <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}