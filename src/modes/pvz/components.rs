use ggez::nalgebra::{
    Point2,
    Vector2,
};
use specs::{
    prelude::*,
    Component,
};

#[derive(Clone, Debug, Component)]
pub struct Sun {
    pub land_y: f32,
}
