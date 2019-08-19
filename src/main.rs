use ggez::*;
use ggez::input::mouse::MouseButton;
use specs::prelude::*;
use tiled::PropertyValue;

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
        if button == MouseButton::Left && *self.world.read_resource::<PlayState>() == PlayState::Play {
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

        if *self.world.read_resource::<PlayState>() == PlayState::Play {
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
            Read<PlayState>,
        ) = self.world.system_data();
        let (transforms, drawables, shooters, grid, play_state) = system_data;

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

        // Highlight the grid cell the mouse is hovering over.
        let mouse_pos = input::mouse::position(ctx);
        let (cell_x, cell_y) = ((mouse_pos.x / grid.cell_size) as u32, (mouse_pos.y / grid.cell_size) as u32);
        if let Some(cell) = grid.get_cell(cell_x, cell_y) {
            let color = if cell == GridCell::Buildable {
                graphics::Color::from_rgba(0, 0, 127, 127)
            } else {
                graphics::Color::from_rgba(127, 0, 0, 127)
            };
            let mesh = graphics::Mesh::new_rectangle(
                ctx,
                graphics::DrawMode::fill(),
                graphics::Rect::new(cell_x as f32 * grid.cell_size, cell_y as f32 * grid.cell_size, grid.cell_size, grid.cell_size),
                color,
            )?;
            graphics::draw(ctx, &mesh, graphics::DrawParam::default())?;
        }

        match *play_state {
            PlayState::Win => {
                graphics::draw(
                    ctx,
                    &graphics::Text::new("VICTORY ACHIEVED!"),
                    graphics::DrawParam::default()
                        .dest([10.0, 10.0]),
                )?;
            }
            PlayState::Lose => {
                graphics::draw(
                    ctx,
                    &graphics::Text::new("Hey. I'm sorry to tell you this. You died."),
                    graphics::DrawParam::default()
                        .dest([10.0, 10.0]),
                )?;
            }
            _ => {}
        }

        graphics::present(ctx)?;
        Ok(())
    }
}

impl<'a, 'b> State<'a, 'b> {
    fn new(_ctx: &mut Context) -> GameResult<Self> {
        // Set up the specs world.
        let mut world = World::new();
        world.register::<Transform>();
        world.register::<Drawable>();
        // Currently the Projectile resource isn't accessed by any systems so it needs to be registered here.
        world.register::<Projectile>();

        let mut dispatcher = DispatcherBuilder::new()
            .with(EnemyAi, "enemy_ai", &[])
            .with(ShooterSystem, "shooter_system", &["enemy_ai"])
            .with(UpdatePosition, "update_position", &["shooter_system"])
            .with(CollisionSystem, "collision_system", &["update_position"])
            .with(AttackSystem, "attack_system", &["collision_system"])
            .with(SpawnerSystem, "spawner_system", &["attack_system"])
            .with(DeathSystem, "death_system", &["spawner_system"])
            .with(WinSystem, "win_system", &["death_system"])
            .build();

        dispatcher.setup(&mut world);

        // Load the grid and entities from Tiled map file.
        // TODO: Use ggez's filesystem::open instead of File::open to support wasm.
        let map = tiled::parse_file("assets/levels/test.tmx")
            .expect("Could not parse level");

        // Initialize Grid from Grid layer.
        let mut grid = Grid::new(map.width, map.height, map.tile_width as f32);
        // TODO: Don't hard-code the layer. Check the name at least.
        for (j, row) in map.layers[0].tiles.iter().enumerate() {
            for (i, tile) in row.iter().enumerate() {
                match tile {
                    1 => { grid.set_cell(i as u32, j as u32, GridCell::Buildable); }
                    2 => { grid.set_cell(i as u32, j as u32, GridCell::Walkable); }
                    _ => {}
                }
            }
        }

        // Iterate over objects. Create Waypoints, Spawners, and Bases.
        for object in &map.object_groups[0].objects {
            let obj_type = if let (true, Some(tileset)) = (object.obj_type.is_empty(), map.get_tileset_by_gid(object.gid)) {
                // Get default value from tileset.
                let tile_id = object.gid - tileset.first_gid;
                let tile = &tileset.tiles[tile_id as usize];
                if let Some(tile_type) = &tile.tile_type {
                    tile_type.as_ref()
                } else {
                    ""
                }
            } else {
                object.obj_type.as_ref()
            };
            let (x, y) = (object.x + object.width as f32 / 2.0,
                          object.y + object.height as f32 / 2.0);
            let (cell_x, cell_y) = ((x / grid.cell_size) as u32, (y / grid.cell_size) as u32);
            match obj_type {
                "base" => {
                    if let Some(PropertyValue::IntValue(waypoint_id)) = object.properties.get("waypoint_id") {
                        world.create_entity()
                            .with(Base {})
                            .with(Waypoint {id: *waypoint_id as u8})
                            .with(Transform::new(x, y))
                            .with(Drawable::Base)
                            .with(Faction::Player)
                            .with(Health { current_hp: 1 })
                            .with(Collider::new(40.0, 40.0))
                            .build();
                        grid.set_cell(cell_x, cell_y, GridCell::Occupied);
                    } else {
                        panic!("Could not find waypoint_id property for base");
                    }
                }
                "spawner" => {
                    world.create_entity()
                        .with(Spawner::default())
                        .with(Transform::new(x, y))
                        .with(Drawable::Spawner)
                        .build();
                    grid.set_cell(cell_x, cell_y, GridCell::Occupied);
                }
                "waypoint" => {
                    if let Some(PropertyValue::IntValue(waypoint_id)) = object.properties.get("waypoint_id") {
                        world.create_entity()
                            .with(Waypoint {id: *waypoint_id as u8})
                            .with(Transform::new(x, y))
                            .with(Drawable::Waypoint)
                            .build();
                        grid.set_cell(cell_x, cell_y, GridCell::Occupied);
                    } else {
                        panic!("Could not find waypoint_id property for base");
                    }
                }
                // Warn since this is an unknown object type.
                obj_type => println!("Warning: Ignoring object of unknown type \"{}\"", obj_type),
            }
        }

        // Insert initial resources.
        world.insert(grid);

        Ok(Self {
            world,
            dispatcher,
        })
    }
}

fn main() -> GameResult {
    let (mut ctx, mut event_loop) = ContextBuilder::new("isengard_returns", "studio_giblets")
        .add_resource_path("./assets")
        .window_setup(conf::WindowSetup::default().title("Isengard Returns!"))
        .build()?;
    let mut state = State::new(&mut ctx)?;
    event::run(&mut ctx, &mut event_loop, &mut state)
}
