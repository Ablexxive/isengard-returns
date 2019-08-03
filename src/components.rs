use specs::prelude::*;

#[derive(Debug)]
pub struct Transform {
    pub x: f32,
    pub y: f32,
}

impl Component for Transform {
    type Storage = VecStorage<Self>;
}
