use ggez::nalgebra::{
    Point2,
    Vector2,
};
use specs::{
    prelude::*,
    Component,
};

#[derive(Default)]
pub struct DeltaTime(pub f32);

#[derive(Clone, Copy, Debug, Component)]
pub struct Transform {
    pub position: Point2<f32>,
}

impl Transform {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            position: Point2::new(x, y),
        }
    }
}

#[derive(Clone, Debug, Component)]
pub enum Drawable {
    Tower,
    Enemy,
    Projectile,
    Spawner,
}

#[derive(Clone, Debug, Component)]
pub struct Shooter {
    pub seconds_per_attack: f32,
    pub cooldown: f32,
    pub attack_radius: f32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Component)]
pub enum Faction {
    Player,
    Enemy,
}

#[derive(Clone, Debug, Component)]
pub struct Velocity(pub Vector2<f32>);

impl Velocity {
    pub fn new(x: f32, y:f32) -> Self {
        Self(Vector2::new(x, y))
    }
}

#[derive(Clone, Debug, Component)]
pub struct Collider {
    pub width: f32,
    pub height: f32,
}

impl Collider {
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            width,
            height,
        }
    }
}

#[derive(Clone, Debug, Component)]
pub struct Projectile {}

#[derive(Clone, Debug, Component)]
pub struct Attacker {
    pub damage: u32,
}

#[derive(Clone, Debug, Component)]
pub struct Health {
    pub current_hp: u32,
}

#[derive(Clone, Debug, Component)]
pub struct Spawner {
    pub spawn_faction: Faction,
    pub spawn_drawable: Drawable,
    pub count: u32,
    pub seconds_to_spawn: f32,
    pub cooldown: f32,
}

impl Spawner {
    pub fn default() -> Self {
        Self {
            spawn_faction: Faction::Enemy,
            spawn_drawable: Drawable::Enemy,
            count: 5,
            seconds_to_spawn: 3.0,
            cooldown: 0.0,
        }
    }
}
