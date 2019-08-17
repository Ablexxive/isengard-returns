use specs::Entity;

#[derive(Clone, Copy, Debug, Default)]
pub struct DeltaTime(pub f32);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlayState {
    Play,
    Win,
    Lose,
}

impl Default for PlayState {
    fn default() -> Self {
        Self::Play
    }
}

pub struct CollisionEvent {
    pub entity_a: Entity,
    pub entity_b: Entity,
}

pub struct DeathEvent {
    pub entity: Entity,
}
