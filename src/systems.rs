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
