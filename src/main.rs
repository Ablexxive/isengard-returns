use ggez::*;
use specs::prelude::*;

use components::*;
use systems::*;

mod components;
mod systems;

struct State<'a, 'b> {
    world: World,
    dispatcher: Dispatcher<'a, 'b>,
}

impl<'a, 'b> ggez::event::EventHandler for State<'a, 'b> {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        {
            // Sets the time and updates it
            let mut delta = self.world.write_resource::<DeltaTime>();
            let duration = timer::duration_to_f64(timer::delta(ctx));
            *delta = DeltaTime(duration as f32);
        }

        self.dispatcher.dispatch(&mut self.world);
        self.world.maintain();
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, graphics::BLACK);

        let system_data: (ReadStorage<Transform>, ReadStorage<Drawable>) = self.world.system_data();
        let (transforms, drawables) = system_data;

        for (transform, drawable) in (&transforms, &drawables).join() {
            let mesh = match drawable {
                Drawable::Tower => {
                    graphics::Mesh::new_circle(
                        ctx,
                        graphics::DrawMode::fill(),
                        mint::Point2{x: 0.0, y: 0.0},
                        20.0,
                        0.1,
                        graphics::WHITE,
                    )?
                },
                Drawable::Enemy => {
                    graphics::Mesh::new_rectangle(
                        ctx,
                        graphics::DrawMode::fill(),
                        graphics::Rect::new_i32(-20, -20, 40, 40),
                        graphics::Color::from_rgb(255, 0, 0),
                    )?
                },
            };

            graphics::draw(ctx, &mesh, graphics::DrawParam::default().dest(transform.position))?;
        }

        graphics::present(ctx)?;
        Ok(())
    }
}

impl<'a, 'b> State<'a, 'b> {
    fn new() -> Self {
        let mut world = World::new();
        world.register::<Transform>();
        world.register::<Drawable>();
        let mut dispatcher = DispatcherBuilder::new()
            .with(UpdatePosition, "update_position", &[])
            .build();

        dispatcher.setup(&mut world);

        // Towers
        for idx in 1..3 {
            let idx = idx as f32;
            world.create_entity()
                .with(Transform::new(idx*50.0, idx*100.0))
                .with(Drawable::Tower)
                .build();
        }

        // Enemy
        world.create_entity()
            .with(Transform::new(0.0, 0.0))
            .with(Velocity { x: 5.0, y: 5.0 })
            .with(Drawable::Enemy)
            .build();

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
