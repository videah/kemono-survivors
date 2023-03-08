use std::fmt::format;
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
        whip_min_damage: 4.0,
        whip_max_damage: 8.0,
    });
}

pub fn spawn_player(mut commands: Commands, asset_server: Res<AssetServer>) {
    let aura: Handle<Image> = asset_server.load("aura.png");

    commands.spawn((
        Player,
        LookAt,
        Health(100.0),
        MovementDirection(Vec2::ZERO),
        WhipWeapon,
        SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.8, 0.7, 0.6),
                custom_size: Some(Vec2::new(32.0, 32.0)),
                ..Default::default()
            },
            ..Default::default()
        },
    )).with_children(
        |parent| {
            parent.spawn((
                AuraEffect,
                SpriteBundle {
                    texture: aura,
                    sprite: Sprite {
                        custom_size: Some(Vec2::new(250.0, 250.0)),
                        ..Default::default()
                    },
                    ..Default::default()
                }
            ));
        }
    );
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
            }
            dir if dir.x > 0.0 => {
                weights[2] -= WEIGHT_FACTOR;
                weights[3] += WEIGHT_FACTOR;
            }
            dir if dir.y > 0.0 => {
                weights[0] += WEIGHT_FACTOR;
                weights[1] -= WEIGHT_FACTOR;
            }
            dir if dir.y < 0.0 => {
                weights[0] -= WEIGHT_FACTOR;
                weights[1] += WEIGHT_FACTOR;
            }
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
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
    mut config: ResMut<WeaponConfig>,
    mut set: ParamSet<(
        Query<(&mut Transform, &mut Health, With<Enemy>)>,
        Query<(&mut Transform, With<WhipWeapon>)>,
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
            if distance < 150.0 {
                // Whip the enemy, knocking it back a bit based on its distance and angle from the player.
                let direction = (enemy_pos - player_pos).normalize();
                let angle = direction.y.atan2(direction.x);
                let knockback_distance = (200.0 - distance) / 100.0 * 25.0; // Adjust the 50.0 value as needed.
                let knockback_vector = Vec2::new(angle.cos(), angle.sin()) * knockback_distance;
                enemy.translation += Vec3::new(knockback_vector.x, knockback_vector.y, 0.0);

                // Damage the enemy between the min and max damage values.
                let damage = rand::thread_rng().gen_range(config.whip_min_damage..config.whip_max_damage).round();
                health.0 -= damage;

                // Spawn a damage indicator
                commands.spawn((
                    DamageIndicator {
                        damage,
                        timer: Timer::from_seconds(2.0, TimerMode::Once),
                    },
                    Text2dBundle {
                        transform: Transform {
                            translation: enemy_pos,
                            ..default()
                        },
                        text: Text::from_section(
                            format!("-{}", damage),
                            TextStyle {
                                font: asset_server.load("FiraSans-Bold.ttf"),
                                font_size: 32.0,
                                color: Color::rgb(0.8, 0.0, 0.0),
                                ..default()
                            },
                        ),
                        ..default()
                    }
                ));
            }
        }
    }
}

/// Remove entities that have 0 health.
pub fn remove_dead(
    mut commands: Commands,
    mut query: Query<(Entity, &Health, &Transform), Without<Player>>,
) {
    for (entity, health, pos) in query.iter_mut() {
        if health.0 <= 0.0 {
            // Remove the entity and spawn an XP orb.
            commands.entity(entity).despawn();
            commands.spawn((
                XpOrb(10.0),
                SpriteBundle {
                    sprite: Sprite {
                        color: Color::rgb(0.0, 0.0, 0.4),
                        custom_size: Some(Vec2::new(8.0, 8.0)),
                        ..Default::default()
                    },
                    transform: Transform {
                        translation: Vec3::new(pos.translation.x, pos.translation.y, 0.0),
                        ..Default::default()
                    },
                    ..Default::default()
                }
            ));
        }
    }
}

pub fn mark_xp_orbs(
    mut commands: Commands,
    mut orbs: Query<(&mut Transform, Entity, With<XpOrb>, Without<Player>)>,
    player: Query<(&Transform, With<Player>)>,
) {
    // If an XP orb is close to the player, mark it as collecting.
    let player_pos = match player.get_single() {
        Ok(player) => player.0.translation,
        Err(e) => {
            info!("No player found: {:?}", e);
            return;
        }
    };

    for (orb, entity, _, _) in orbs.iter_mut() {
        let orb_pos = orb.translation;
        let distance = (orb_pos - player_pos).length();
        if distance < 100.0 {
            // Mark the orb as being collected.
            commands.entity(entity).insert(
                Collecting {}
            );
        }
    }
}

pub fn collect_items(
    mut commands: Commands,
    time: Res<Time>,
    mut items: Query<(&mut Transform, Entity, With<Collecting>, Without<Player>)>,
    mut player: Query<(&Transform, With<Player>)>,
) {
    // Move collecting items towards the player.
    let player_pos = match player.get_single_mut() {
        Ok(player) => player.0.translation,
        Err(e) => {
            info!("No player found: {:?}", e);
            return;
        }
    };

    for (mut item, entity, _, _) in items.iter_mut() {
        let direction = (player_pos - item.translation).normalize();
        let distance = (item.translation - player_pos).length();

        // Calculate move_speed based on distance from player
        // Make sure it's never slower than the player's move speed.
        let move_speed = (distance / 10.0).max(1.0) * 200.0;

        let move_delta = (direction * move_speed) * time.delta_seconds();
        item.translation += move_delta;

        // If the item is close enough to the player, despawn it.
        let distance = (item.translation - player_pos).length();
        if distance < 10.0 {
            commands.entity(entity).despawn();
        }
    }
}

pub fn strobe_aura(
    config: Res<WeaponConfig>,
    mut query: Query<(&mut Sprite, With<AuraEffect>)>,
) {
    for (mut sprite, _) in query.iter_mut() {
        let percent = config.whip_timer.percent();
        // let scale = (time * 10.0).sin() * 0.5 + 0.5;

        // Have the aura scale up and down.
        // When percent is zero the aura should be going down
        // When the percent is 50 the aura should be going up
        // When the percent is 100 the aura should be going down
        let scale = if percent < 0.5 {
            // Going down
            percent * 2.0
        } else {
            // Going up
            (1.0 - percent) * 2.0
        };

        // Invert the scale so that the aura is smaller when the whip is charging.
        let scale = 1.0 - scale;

        sprite.color.set_a((scale - 0.5).max(0.1));
    }
}

pub fn animate_damage_indicators(
    mut commands: Commands,
    mut query: Query<(&mut Transform, &mut Text, Entity, &mut DamageIndicator)>,
    time: Res<Time>,
) {
    for (mut transform, mut text, entity, mut indicator) in query.iter_mut() {
        indicator.timer.tick(time.delta());

        if indicator.timer.finished() {
            // Remove the entity if the timer is finished.
            commands.entity(entity).despawn();
        } else {
            // Move the indicator up.
            transform.translation.y += 100.0 * time.delta_seconds();

            // Move the indicator side to side.
            let percent = indicator.timer.percent();
            let scale = (percent * 10.0).sin() * 0.5 + 0.5;
            transform.translation.x += (scale - 0.5) * 100.0 * time.delta_seconds();

            // Fade the indicator out.
            let percent = indicator.timer.percent();
            let alpha = 1.0 - percent;
            text.sections[0].style.color.set_a(alpha);
        }
    }
}