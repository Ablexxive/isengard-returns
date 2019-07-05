use ggez::*;

struct State {}

impl ggez::event::EventHandler for State {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        Ok(())
    }
    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        Ok(())
    }
}

fn main() {
    let state = &mut State { };

    let c = conf::Conf::new();
    let (ref mut ctx, ref mut event_loop) = ContextBuilder::new("isengard_returns", "studio_giblets")
        .conf(c)
        .build()
        .unwrap();
    event::run(ctx, event_loop, state).unwrap();
}

