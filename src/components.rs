use specs::{
    prelude::*,
    Component
};

#[derive(Clone, Copy, Debug, Component)]
#[storage(VecStorage)]
pub struct Transform {
    pub x: f32,
    pub y: f32,
}
