use mint::{Point2};
use ggez::*;
use ggez::graphics;
//use ggez::{timer, ContextBuilder, Context, GameResult, conf, event};

struct State {
    dt: std::time::Duration,
    pos_x: f32,
    pos_y: f32,
}

impl ggez::event::EventHandler for State {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        self.dt = timer::delta(ctx);
        self.pos_x = self.pos_x + 0.1;
        self.pos_y = self.pos_y + 0.1;
        Ok(())
    }
    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, graphics::BLACK);
        // Drawing a circle
        let circle = graphics::Mesh::new_circle(
            ctx,
            graphics::DrawMode::fill(),
            Point2{x: self.pos_x, y: self.pos_y},
            100.0,
            0.1,
            graphics::WHITE,
            )?;
        graphics::draw(ctx, &circle, graphics::DrawParam::default())?;

        // Add some text
        //graphics::Text::new(

        graphics::draw(
            ctx,
            &graphics::Text::new(format!("{:?}", self.dt)),
            graphics::DrawParam::default()
                .dest([10.0, 10.0]),
        )?;


        graphics::present(ctx)?;
        println!("Hello ggez dt = {}ns", self.dt.subsec_nanos());
        Ok(())
    }
}

fn main() {
    let state = &mut State {
        dt: std::time::Duration::new(0,0),
        pos_x: 200.0,
        pos_y: 300.0,
    };

    let c = conf::Conf::new();
    let (ref mut ctx, ref mut event_loop) = ContextBuilder::new("isengard_returns", "studio_giblets")
        .conf(c)
        .build()
        .unwrap();
    event::run(ctx, event_loop, state).unwrap();
}

