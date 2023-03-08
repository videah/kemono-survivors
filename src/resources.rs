use bevy::prelude::*;

#[derive(Resource)]
pub struct EnemySpawnConfig {
    /// How often to spawn a new enemy? (repeating timer)
    pub timer: Timer,
}

#[derive(Resource)]
pub struct WeaponConfig {
    /// How often to attack with whip? (repeating timer)
    pub whip_timer: Timer,
    pub whip_min_damage: f32,
    pub whip_max_damage: f32,
}