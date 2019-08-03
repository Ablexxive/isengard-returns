use ggez::*;
use specs::prelude::*;

mod components;
use components::Transform;
mod systems;
use systems::ShowPosition;

struct State {}

impl ggez::event::EventHandler for State {
    fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {
        Ok(())
    }
    fn draw(&mut self, _ctx: &mut Context) -> GameResult<()> {
        Ok(())
    }
}

fn main() {
    let mut world = World::new();
    let mut dispatcher = DispatcherBuilder::new()
        .with(ShowPosition, "show_position", &[])
        .build();

    dispatcher.setup(&mut world);

    for idx in 0..10 {
        let idx = idx as f32;
        world.create_entity().with(Transform { x: idx, y: idx*2.0 }).build();
    }

    dispatcher.dispatch(&mut world);
    world.maintain();

    // State via ggez
    let state = &mut State { };

    let config = conf::Conf::new();
    let (ref mut ctx, ref mut event_loop) = ContextBuilder::new("isengard_returns", "studio_giblets")
        .conf(config)
        .build()
        .unwrap();
    event::run(ctx, event_loop, state).unwrap();
}

