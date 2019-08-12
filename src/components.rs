use ggez::nalgebra::{Point2, Vector2};
use specs::{
    prelude::*,
    Component
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
