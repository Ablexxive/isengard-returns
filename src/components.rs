use ggez::mint::Point2;
use specs::{
    prelude::*,
    Component
};

#[derive(Clone, Copy, Debug, Component)]
pub struct Transform {
    pub position: Point2<f32>,
}

impl Transform {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            position: Point2 { x, y },
        }
    }
}

#[derive(Clone, Debug, Component)]
pub enum Drawable {
    Tower,
    Enemy,
}

#[derive(Clone, Debug, Component)]
pub struct Shooter {
}
