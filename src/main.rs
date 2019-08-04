use ggez::*;
use mint::Point2;
use specs::prelude::*;

use components::Transform;
//use systems::ShowPosition;

mod components;
mod systems;

struct State<'a, 'b> {
    world: World,
    dispatcher: Dispatcher<'a, 'b>,
}

impl<'a, 'b> ggez::event::EventHandler for State<'a, 'b> {
    fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {
        self.dispatcher.dispatch(&mut self.world);
        self.world.maintain();
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, graphics::BLACK);

        let transforms: (ReadStorage<Transform>) = self.world.system_data();
        for transform in transforms.join() {
            let circle = graphics::Mesh::new_circle(
                ctx,
                graphics::DrawMode::fill(),
                Point2{x: transform.x, y: transform.y},
                20.0,
                0.1,
                graphics::WHITE,
            )?;
            graphics::draw(ctx, &circle, graphics::DrawParam::default())?;
        }

        graphics::present(ctx)?;
        Ok(())
    }
}

impl<'a, 'b> State<'a, 'b> {
    fn new() -> Self {
        let mut world = World::new();
        world.register::<Transform>();
        let mut dispatcher = DispatcherBuilder::new()
            //.with(ShowPosition, "show_position", &[])
            .build();

        dispatcher.setup(&mut world);

        // Tmp Enteties
        for idx in 0..10 {
            let idx = idx as f32;
            world.create_entity().with(Transform { x: idx*50.0, y: idx*100.0 }).build();
        }

        Self {
            world,
            dispatcher,
        }
    }

}

fn main() {

    // State via ggez
    let mut state = State::new();

    let config = conf::Conf::new();
    let (ref mut ctx, ref mut event_loop) = ContextBuilder::new("isengard_returns", "studio_giblets")
        .conf(config)
        .build()
        .unwrap();
    event::run(ctx, event_loop, &mut state).unwrap();
}
