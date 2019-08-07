use ggez::nalgebra;
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
        let (dt, mut transforms, vel) = data;

        for (transform, vel) in (&mut transforms, &vel).join() {
            transform.position.x += dt.0*vel.x;
            transform.position.y += dt.0*vel.y;
        }
    }
}

pub struct ShooterSystem;

impl<'a> System<'a> for ShooterSystem {
    type SystemData = (Read<'a, DeltaTime>,
                       ReadStorage<'a, Transform>,
                       WriteStorage<'a, Shooter>,
                       ReadStorage<'a, Faction>);

    fn run(&mut self, data: Self::SystemData) {
        let (dt, transforms, mut shooters, factions) = data;

        for (transform, shooter,  faction) in (&transforms, &mut shooters, &factions).join() {
            if shooter.cooldown > 0.0{
                shooter.cooldown -= dt.0;
            } else {
                for (target_transform, target_fraction) in (&transforms, &factions) .join(){
                    if target_fraction != faction {
                        let distance = nalgebra::distance(&transform.position, &target_transform.position);
                        if distance <= shooter.attack_radius {
                            // TODO: Spawn a projectile

                            shooter.cooldown = shooter.seconds_per_attack;
                            println!("Inner loop - found an enemy");
                            break;
                        }
                    }
                }
            }
        }
    }

}
