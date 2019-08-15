use specs::Entity;

#[derive(Default)]
pub struct DeltaTime(pub f32);

#[derive(Default)]
pub struct YouLose(pub bool);

pub struct CollisionEvent {
    pub entity_a: Entity,
    pub entity_b: Entity,
}

pub struct DeathEvent {
    pub entity: Entity,
}
