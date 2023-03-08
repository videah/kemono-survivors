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