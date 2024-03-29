use ggez::nalgebra;
use specs::prelude::*;

use crate::components::*;
use crate::rect::*;
use crate::resources::*;

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
            if shooter.cooldown > 0.0 {
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
                            let velocity = direction * 300.0;
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

pub struct CollisionSystem;

impl<'a> System<'a> for CollisionSystem {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, Transform>,
        ReadStorage<'a, Faction>,
        ReadStorage<'a, Collider>,
        Write<'a, Vec<CollisionEvent>>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (ents, transforms, factions, colliders, mut collision_events) = data;

        // Each collisions will generate two events, going from enity A -> B and B -> A
        for (ent, transform, faction, collider) in (&ents, &transforms, &factions, &colliders).join() {
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
        Write<'a, Vec<DeathEvent>>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (collision_events, entities, attackers, mut healths, mut death_events) = data;

        for event in collision_events.iter() {
            let attacker = attackers.get(event.entity_a);
            let health = healths.get_mut(event.entity_b);

            if let (Some(attacker), Some(health)) = (attacker, health) {
                health.current_hp = health.current_hp.saturating_sub(attacker.damage);

                // TODO: Don't delete all attackers
                if let Err(e) = entities.delete(event.entity_a) {
                    println!("Entity could not be deleted {}", e);
                } else {
                    death_events.push(DeathEvent {
                        entity: event.entity_a
                    });
                }

                if health.current_hp == 0 {
                    if let Err(e) = entities.delete(event.entity_b) {
                        println!("Entity could not be deleted {}", e);
                    } else {
                        death_events.push(DeathEvent {
                            entity: event.entity_b
                        });
                    }
                }
            }
        }
    }
}

pub struct SpawnerSystem;

impl<'a> System<'a> for SpawnerSystem {
    type SystemData = (
        Read<'a, DeltaTime>,
        Entities<'a>,
        Read<'a, LazyUpdate>,
        ReadStorage<'a, Transform>,
        WriteStorage<'a, Spawner>,
        ReadStorage<'a, Waypoint>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (dt, entities, lazy, transforms, mut spawners, waypoints) = data;


        for (transform, spawner, spawner_ent) in (&transforms, &mut spawners, &entities).join() {
            if spawner.cooldown > 0.0 {
                spawner.cooldown -= dt.0;
            } else {
                // Get initial waypoint entity
                let waypoint_entity = {
                    let mut entity = None;
                    for (ent, waypoint) in (&entities, &waypoints).join() {
                        if waypoint.id == 0 {
                            entity = Some(ent);
                        }
                    }
                    entity
                }.expect("Waypoint 0 does not exist");


                spawner.cooldown = spawner.seconds_to_spawn;
                // Spawn entity
                let new_ent = entities.create();
                lazy.insert(new_ent, Enemy { current_waypoint: waypoint_entity});
                lazy.insert(new_ent, *transform);
                lazy.insert(new_ent, spawner.spawn_faction);
                lazy.insert(new_ent, spawner.spawn_drawable);
                lazy.insert(new_ent, Velocity::new(60.0, 0.0));
                lazy.insert(new_ent, Collider::new(40.0, 40.0));
                lazy.insert(new_ent, Health {current_hp: 5});
                lazy.insert(new_ent, Attacker {damage:1});

                spawner.count -= 1;
                if spawner.count == 0 {
                    if let Err(e) = entities.delete(spawner_ent) {
                        println!("Entity could not be deleted {}", e);
                    }
                }
            }
        }
    }
}

pub struct EnemyAi;

impl<'a> System<'a> for EnemyAi {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, Waypoint>,
        ReadStorage<'a, Transform>,
        WriteStorage<'a, Enemy>,
        WriteStorage<'a, Velocity>,
    );

    fn run (&mut self, data: Self::SystemData) {
        let (entities, waypoints, transforms, mut enemies, mut velocities) = data;

        for (enemy, transform, velocity) in (&mut enemies, &transforms, &mut velocities).join() {
            // Update which waypoint an enemy is heading towards
            let mut waypoint_transform = transforms.get(enemy.current_waypoint)
                .expect("Waypoint doesn't have a transform?");
            let distance = nalgebra::distance(&transform.position, &waypoint_transform.position);

            if distance <= 10.0 {
                let new_waypoint_id = waypoints.get(enemy.current_waypoint).expect("Enemy doesn't have a waypoint. Boo.")
                    .id + 1;

                let waypoint_entity = {
                    let mut entity = None;
                    for (ent, waypoint) in (&entities, &waypoints).join() {
                        if waypoint.id == new_waypoint_id {
                            entity = Some(ent);
                        }
                    }
                    entity
                }.expect(&format!("Waypoint {} does not exist", new_waypoint_id));
                waypoint_transform = transforms.get(waypoint_entity)
                    .expect("Really this shouldn't happen?");
                enemy.current_waypoint = waypoint_entity;
            }

            // Now update velocity
            let speed = velocity.0.magnitude();
            let direction = (waypoint_transform.position - transform.position).normalize();
            velocity.0 = speed * direction;
        }
    }
}

pub struct DeathSystem;

impl<'a> System<'a> for DeathSystem {
    type SystemData = (
        ReadStorage<'a, Base>,
        ReadStorage<'a, Enemy>,
        Read<'a, Vec<DeathEvent>>,
        Write<'a, BuildResources>,
        Write<'a, PlayState>,
    );

    fn run (&mut self, data: Self::SystemData) {
        let (bases, enemies, death_events, mut build_resources, mut play_state) = data;

        for death in death_events.iter() {
            if let Some(_base) = bases.get(death.entity) {
                *play_state = PlayState::Lose;
            }
            if let Some(_enemy) = enemies.get(death.entity) {
                // TODO: Make this tunable in data somehow.
                build_resources.bits += 5;
            }
        }
    }
}

pub struct WinSystem;

impl<'a> System<'a> for WinSystem {
    type SystemData = (
        ReadStorage<'a, Base>,
        ReadStorage<'a, Enemy>,
        ReadStorage<'a, Spawner>,
        Write<'a, PlayState>,
    );

    fn run (&mut self, data: Self::SystemData) {
        let (bases, enemies, spawners, mut play_state) = data;

        // Player wins When all enemies and spawners are gone and there are still bases alive.
        if !bases.is_empty() && enemies.is_empty() && spawners.is_empty() {
            *play_state = PlayState::Win;
        }
    }
}
