use ggez::nalgebra;
use specs::prelude::*;

use crate::components::*;
use crate::rect::*;

pub struct UpdatePosition;

impl<'a> System<'a> for UpdatePosition {
    type SystemData = (
        Read<'a, DeltaTime>,
        WriteStorage<'a, Transform>,
        ReadStorage<'a, Velocity>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (dt, mut transforms, vel) = data;

        for (transform, vel) in (&mut transforms, &vel).join() {
            transform.position += dt.0*vel.0;
        }
    }
}

pub struct ShooterSystem;

impl<'a> System<'a> for ShooterSystem {
    type SystemData = (
        Read<'a, DeltaTime>,
        Entities<'a>,
        Read<'a, LazyUpdate>,
        ReadStorage<'a, Transform>,
        WriteStorage<'a, Shooter>,
        ReadStorage<'a, Faction>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (dt, ent, lazy, transforms, mut shooters, factions) = data;

        for (transform, shooter,  faction) in (&transforms, &mut shooters, &factions).join() {
            if shooter.cooldown > 0.0{
                shooter.cooldown -= dt.0;
            } else {
                for (target_transform, target_fraction) in (&transforms, &factions).join() {
                    if target_fraction != faction {
                        // Determine if enemy is within range of the tower
                        let distance = nalgebra::distance(&transform.position, &target_transform.position);
                        if distance <= shooter.attack_radius {
                            shooter.cooldown = shooter.seconds_per_attack;
                            // Spawning the projectile
                            let projectile = ent.create();
                            lazy.insert(projectile, Projectile{});
                            lazy.insert(projectile, *transform);
                            lazy.insert(projectile, Drawable::Projectile);
                            lazy.insert(projectile, *faction);

                            let direction = (target_transform.position - transform.position).normalize();
                            let velocity = direction * 100.0;
                            lazy.insert(projectile, Velocity(velocity));

                            // TODO - Have it collide
                            lazy.insert(projectile, Collider::new(8.0, 8.0));


                            println!("Inner loop - found an enemy");
                            break;
                        }
                    }
                }
            }
        }
    }
}

pub struct CollisionSystem;

impl<'a> System<'a> for CollisionSystem {
    type SystemData = (
        ReadStorage<'a, Projectile>,
        ReadStorage<'a, Transform>,
        ReadStorage<'a, Faction>,
        ReadStorage<'a, Collider>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (projectiles, transforms, factions, colliders) = data;

        for (_projectile, transform, faction, collider) in (&projectiles, &transforms, &factions, &colliders).join() {
            let rect = Rect {
                x: transform.position.x,
                y: transform.position.y,
                width: collider.width,
                height: collider.height,
            };

            for (target_transform, target_faction, target_collider) in (&transforms, &factions, &colliders).join() {
                if faction != target_faction {
                    let target_rect = Rect {
                        x: target_transform.position.x,
                        y: target_transform.position.y,
                        width: target_collider.width,
                        height: target_collider.height,
                    };
                    if rect.overlaps(&target_rect) {
                        println!("Collides!");
                    }
                }
            }
        }
    }
}
