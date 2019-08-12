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

                            lazy.insert(projectile, Collider::new(8.0, 8.0));
                            lazy.insert(projectile, Attacker {damage: 1});
                            break;
                        }
                    }
                }
            }
        }
    }
}

pub struct CollisionEvent {
    entity_a: Entity,
    entity_b: Entity,
}

pub struct CollisionSystem;

impl<'a> System<'a> for CollisionSystem {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, Projectile>,
        ReadStorage<'a, Transform>,
        ReadStorage<'a, Faction>,
        ReadStorage<'a, Collider>,
        Write<'a, Vec<CollisionEvent>>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (ents, projectiles, transforms, factions, colliders, mut collision_events) = data;

        for (_projectile, ent, transform, faction, collider) in (&projectiles, &ents, &transforms, &factions, &colliders).join() {
            let rect = Rect {
                x: transform.position.x,
                y: transform.position.y,
                width: collider.width,
                height: collider.height,
            };

            for (target_ent, target_transform, target_faction, target_collider) in (&ents, &transforms, &factions, &colliders).join() {
                if faction != target_faction {
                    let target_rect = Rect {
                        x: target_transform.position.x,
                        y: target_transform.position.y,
                        width: target_collider.width,
                        height: target_collider.height,
                    };
                    if rect.overlaps(&target_rect) {
                        let event = CollisionEvent { entity_a: ent, entity_b: target_ent };
                        collision_events.push(event);
                    }
                }
            }
        }
    }
}

pub struct AttackSystem;

impl<'a> System<'a> for AttackSystem {
    type SystemData = (
        Read<'a, Vec<CollisionEvent>>,
        Entities<'a>,
        ReadStorage<'a, Attacker>,
        WriteStorage<'a, Health>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (collision_events, entities, attackers, mut healths) = data;

        for event in collision_events.iter() {
            let attacker = attackers.get(event.entity_a);
            let health = healths.get_mut(event.entity_b);

            if let (Some(attacker), Some(health)) = (attacker, health) {
                health.current_hp = health.current_hp.saturating_sub(attacker.damage);

                if let Err(e) = entities.delete(event.entity_a) {
                    println!("Entity could not be deleted {}", e);
                }

                if health.current_hp == 0 {
                    if let Err(e) = entities.delete(event.entity_b) {
                        println!("Entity could not be deleted {}", e);
                    }
                }
            }
        }
    }
}
