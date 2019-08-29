use ggez::nalgebra as na;
use rand::prelude::*;
use specs::prelude::*;

use crate::components::*;
use crate::resources::*;

use super::components::*;

const SUN_SPAWN_TIME_RANGE: [f32; 2] = [1.0, 10.0];
const SUN_START_X_RANGE: [f32; 2] = [50.0, 400.0];
const SUN_START_Y: f32 = -50.0;
const SUN_LAND_Y_RANGE: [f32; 2] = [300.0, 560.0];
const SUN_FALL_SPEED: f32 = 80.0;

pub struct SunSpawnSystem {
    timer: f32,
}

impl SunSpawnSystem {
    pub fn new() -> Self {
        let mut rng = rand::thread_rng();
        Self {
            timer: rng.gen_range(SUN_SPAWN_TIME_RANGE[0], SUN_SPAWN_TIME_RANGE[1]),
        }
    }
}

impl<'a> System<'a> for SunSpawnSystem {
    type SystemData = (
        Read<'a, DeltaTime>,
        Entities<'a>,
        Read<'a, LazyUpdate>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (dt, entities, lazy) = data;

        if self.timer > 0.0 {
            self.timer -= dt.0;
        } else {
            let mut rng = rand::thread_rng();

            // Spawn a sun.
            let entity = entities.create();
            let x = rng.gen_range(SUN_START_X_RANGE[0], SUN_START_X_RANGE[1]);
            let y = SUN_START_Y;
            let land_y = rng.gen_range(SUN_LAND_Y_RANGE[0], SUN_LAND_Y_RANGE[1]);
            let transform = Transform::new(x, y);
            lazy.insert(entity, transform);
            lazy.insert(entity, Drawable::Sun);
            lazy.insert(entity, Velocity(na::Vector2::new(0.0, SUN_FALL_SPEED)));
            lazy.insert(entity, Sun { land_y });
            lazy.insert(entity, Collider::new(30.0, 30.0));

            self.timer = rng.gen_range(SUN_SPAWN_TIME_RANGE[0], SUN_SPAWN_TIME_RANGE[1]);
        }
    }
}

pub struct SunBehaviorSystem;

impl<'a> System<'a> for SunBehaviorSystem {
    type SystemData = (
        ReadStorage<'a, Sun>,
        ReadStorage<'a, Transform>,
        WriteStorage<'a, Velocity>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (suns, transforms, mut velocities) = data;

        for (sun, transform, velocity) in (&suns, &transforms, &mut velocities).join() {
            // TODO: Remove the sun and velocity components?
            if transform.position.y >= sun.land_y {
                velocity.0.y = 0.0;
            }
        }
    }
}
