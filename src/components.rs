use specs::{
    prelude::*,
    Component
};

#[derive(Clone, Copy, Debug, Component)]
pub struct Transform {
    pub x: f32,
    pub y: f32,
}

#[derive(Clone, Debug, Component)]
pub struct Drawable {
}

#[derive(Clone, Debug, Component)]
pub struct Shooter {
}
