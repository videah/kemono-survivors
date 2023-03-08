use bevy::prelude::*;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Component)]
pub struct Player;

#[derive(Component)]
pub struct MainCamera;

#[derive(Component)]
pub struct LookAt;

#[derive(Component)]
pub struct Enemy;

#[derive(Component)]
pub struct WhipWeapon;

#[derive(Component)]
pub struct Health(pub f32);

#[derive(Component)]
pub struct MovementDirection(pub Vec2);

#[derive(Component)]
pub struct XpOrb(pub f32);

#[derive(Component)]
pub struct Collecting;

#[derive(Component)]
pub struct AuraEffect;

#[derive(Component)]
pub struct DamageIndicator {
    pub damage: f32,
    pub timer: Timer,
}