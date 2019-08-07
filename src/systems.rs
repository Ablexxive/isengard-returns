use specs::prelude::*;

use crate::components::*;

pub struct ShowPosition;

impl<'a> System<'a> for ShowPosition {
    type SystemData = ReadStorage<'a, Transform>;

    fn run(&mut self, position: Self::SystemData) {
        for position in position.join() {
            println!("Entity Location: {:?}", &position);
        }
    }
}

pub struct UpdatePosition;

impl<'a> System<'a> for UpdatePosition {
    type SystemData = (Read<'a, DeltaTime>,
                       WriteStorage<'a, Transform>,
                       ReadStorage<'a, Velocity>);

    fn run(&mut self, data: Self::SystemData) {
        let (dt, mut pos, vel) = data;

        for (pos, vel) in (&mut pos, & vel).join() {
            pos.position.x += dt.0*vel.x;
            pos.position.y += dt.0*vel.y;
        }
    }
}
