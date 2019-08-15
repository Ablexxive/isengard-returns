use ggez::*;
use ggez::input::mouse::MouseButton;
use specs::prelude::*;

use components::*;
use grid::*;
use resources::*;
use systems::*;

mod components;
mod grid;
mod rect;
mod resources;
mod systems;

struct State<'a, 'b> {
    world: World,
    dispatcher: Dispatcher<'a, 'b>,
}

impl<'a, 'b> ggez::event::EventHandler for State<'a, 'b> {
    fn mouse_button_down_event(&mut self, _ctx: &mut Context, button: MouseButton, x: f32, y: f32) {
        if button == MouseButton::Left {
            // Check which grid cell we've clicked. If nothing is there, build a tower.
            let world_pos = {
                let mut grid = self.world.write_resource::<Grid>();
                let (cell_x, cell_y) = ((x / grid.cell_size) as u32, (y / grid.cell_size) as u32);
                if grid.is_buildable(cell_x, cell_y) {
                    // FIXME: We might wanna defer this to after the tower actually exists. In case
                    // something goes wrong in creating it.
                    grid.set_cell(cell_x, cell_y, GridCell::Occupied);
                    // Return the center of the cell in world coordinates.
                    Some(((cell_x as f32 * grid.cell_size) + grid.cell_size / 2.0,
                          (cell_y as f32 * grid.cell_size) + grid.cell_size / 2.0))
                } else {
                    None
                }
            };

            // Try to build a tower at the location clicked.
            if let Some((world_x, world_y)) = world_pos {
                self.world.create_entity()
                    .with(Transform::new(world_x, world_y))
                    .with(Drawable::Tower)
                    .with(Faction::Player)
                    .with(Shooter { seconds_per_attack: 1.0, cooldown: 0.0, attack_radius: 100.0 })
                    .build();
                println!("Built tower at {:?}!", (world_x, world_y));
            };
        }
    }

    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        // Call maintain to update all entities created via input events.
        self.world.maintain();

        {
            // Sets the time and updates it
            let mut delta = self.world.write_resource::<DeltaTime>();
            let duration = timer::duration_to_f64(timer::delta(ctx));
            *delta = DeltaTime(duration as f32);

            // Clears collision event vector
            let mut collisions = self.world.write_resource::<Vec<CollisionEvent>>();
            collisions.clear();
            let mut death_events = self.world.write_resource::<Vec<DeathEvent>>();
            death_events.clear();
        }

        if self.world.read_resource::<YouLose>().0 {
            //Intentionally left blank.
        } else {
            self.dispatcher.dispatch(&mut self.world);
            self.world.maintain();
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, graphics::BLACK);

        let system_data: (
            ReadStorage<Transform>,
            ReadStorage<Drawable>,
            ReadStorage<Shooter>,
            Read<Grid>,
            Read<YouLose>,
        ) = self.world.system_data();
        let (transforms, drawables, shooters, grid, you_lose) = system_data;

        // Draw the grid first.
        let grid_mesh = {
            let mut mb = graphics::MeshBuilder::new();
            for j in 0..grid.height {
                for i in 0..grid.width {
                    let (x, y) = (i as f32 * grid.cell_size, j as f32 * grid.cell_size);
                    mb.rectangle(
                        graphics::DrawMode::stroke(2.0),
                        graphics::Rect::new(x, y, grid.cell_size, grid.cell_size),
                        graphics::Color::from_rgb(60, 60, 60),
                    );
                }
            }
            mb.build(ctx)?
        };
        graphics::draw(ctx, &grid_mesh, graphics::DrawParam::default())?;

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
                Drawable::Projectile => {
                    graphics::Mesh::new_circle(
                        ctx,
                        graphics::DrawMode::fill(),
                        mint::Point2{x: 0.0, y: 0.0},
                        5.0,
                        0.1,
                        graphics::Color::from_rgb(0, 0, 255),
                    )?
                },
                Drawable::Spawner => {
                    graphics::Mesh::new_circle(
                        ctx,
                        graphics::DrawMode::fill(),
                        mint::Point2{x: 0.0, y: 0.0},
                        8.0,
                        0.1,
                        graphics::Color::from_rgb(0, 255, 0),
                    )?
                },
                Drawable::Base => {
                    graphics::Mesh::new_rectangle(
                        ctx,
                        graphics::DrawMode::stroke(7.0),
                        graphics::Rect::new_i32(-20, -20, 40, 40),
                        graphics::Color::from_rgb(255, 0, 0),
                    )?
                },
                Drawable::Waypoint => {
                    graphics::Mesh::new_circle(
                        ctx,
                        graphics::DrawMode::stroke(4.0),
                        mint::Point2{x: 0.0, y: 0.0},
                        8.0,
                        0.1,
                        graphics::Color::from_rgb(100, 100, 100),
                    )?
                },
            };

            graphics::draw(ctx, &mesh, graphics::DrawParam::default().dest(transform.position))?;
        }

        for (transform, shooter) in (&transforms, &shooters).join() {
            let mesh = graphics::Mesh::new_circle(
                       ctx,
                       graphics::DrawMode::stroke(3.0),
                       mint::Point2{x: 0.0, y: 0.0},
                       shooter.attack_radius,
                       0.1,
                       graphics::WHITE,
                   )?;
            graphics::draw(ctx, &mesh, graphics::DrawParam::default().dest(transform.position))?;
        }

        if you_lose.0 {
            graphics::draw(
                ctx,
                &graphics::Text::new("Hey. I'm sorry to tell you this. You died."),
                graphics::DrawParam::default()
                    .dest([10.0, 10.0]),
            )?;
        }

        graphics::present(ctx)?;
        Ok(())
    }
}

impl<'a, 'b> State<'a, 'b> {
    fn new() -> Self {
        let grid = Grid::new(20, 20, 40.0);

        let mut world = World::new();
        world.register::<Transform>();
        world.register::<Drawable>();
        // Currently the Projectile resource isn't accessed by any systems so it needs to be registered here.
        world.register::<Projectile>();

        // Insert initial resources.
        world.insert(grid);

        let mut dispatcher = DispatcherBuilder::new()
            .with(EnemyAi, "enemy_ai", &[])
            .with(ShooterSystem, "shooter_system", &["enemy_ai"])
            .with(UpdatePosition, "update_position", &["shooter_system"])
            .with(CollisionSystem, "collision_system", &["update_position"])
            .with(AttackSystem, "attack_system", &["collision_system"])
            .with(SpawnerSystem, "spawner_system", &["attack_system"])
            .with(DeathSystem, "death_system", &["spawner_system"])
            .build();

        dispatcher.setup(&mut world);

        // Generate a level in code (for now). We'll do this via data files soon.

        // Waypoints
        world.create_entity()
            .with(Waypoint {id: 0})
            .with(Transform::new(100.0, 300.0))
            .with(Drawable::Waypoint)
            .build();

        // Enemy Spawner
        world.create_entity()
            .with(Spawner::default())
            .with(Transform::new(0.0, 200.0))
            .with(Drawable::Spawner)
            .build();

        // Base
        world.create_entity()
            .with(Base {})
            .with(Waypoint {id: 1})
            .with(Transform::new(400.0, 200.0))
            .with(Drawable::Base)
            .with(Faction::Player)
            .with(Health { current_hp: 1 })
            .with(Collider::new(40.0, 40.0))
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

    let config = conf::Conf {
        window_setup: conf::WindowSetup::default()
            .title("Isengard Returns!"),
        .. conf::Conf::default()
    };
    let (ref mut ctx, ref mut event_loop) = ContextBuilder::new("isengard_returns", "studio_giblets")
        .conf(config)
        .build()
        .unwrap();
    event::run(ctx, event_loop, &mut state).unwrap();
}
