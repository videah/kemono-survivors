use bevy::prelude::*;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Component)]
pub struct Player;

#[derive(Component)]
pub struct MainCamera;

#[derive(Component)]
pub struct LookAt;

#[derive(Component)]
pub struct Enemy;