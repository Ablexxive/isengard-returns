use std::f32;

use ggez::*;
use ggez::input::{
    keyboard::{KeyCode, KeyMods},
    mouse::MouseButton,
};
use ggez::nalgebra::Point2;
use specs::prelude::*;

use components::*;
use debug_ui::*;
use grid::*;
use rect::Rect;
use resources::*;
use systems::*;

mod components;
mod debug_ui;
mod grid;
mod level;
mod modes;
mod rect;
mod resources;
mod systems;

#[derive(Clone, Debug)]
enum LoadLevelRequest {
    None,
    Reload,
    NewLevel(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum MouseAction {
    Select,
    BuildTower,
}

struct State<'a, 'b> {
    // Game state.
    world: World,
    dispatcher: Dispatcher<'a, 'b>,
    current_level: String,
    level_request: LoadLevelRequest,

    // Place UI state.
    mouse_action: MouseAction,

    debug_ui: DebugUi,
    // Debug UI state.
    show_debug_ui: bool,
    draw_colliders: bool,
    level_list: Vec<String>,
}

impl<'a, 'b> ggez::event::EventHandler for State<'a, 'b> {
    fn mouse_button_down_event(&mut self, _ctx: &mut Context, button: MouseButton, x: f32, y: f32) {
        // Debug UI is taking mouse input, so ignore this click.
        if self.debug_ui.io().want_capture_mouse {
            return;
        }

        let play_state = *self.world.read_resource::<PlayState>();

        if play_state == PlayState::Play {
            match (&self.mouse_action, button) {
                (MouseAction::Select, MouseButton::Left) => {
                    // TODO: If we click on a sun, then pick it up and give us more bits.
                    // TODO: Figure out a more general way to do this. And do it somewhere better.
                    // Maybe a system?
                    let system_data: (
                        Entities,
                        Write<BuildResources>,
                        ReadStorage<Transform>,
                        ReadStorage<Collider>,
                        ReadStorage<modes::pvz::components::Sun>,
                    ) = self.world.system_data();
                    let (entities, mut build_resources, transforms, colliders, suns) = system_data;

                    for (entity, transform, collider, _sun) in (&entities, &transforms, &colliders, &suns).join() {
                        let rect = Rect {
                            x: transform.position.x,
                            y: transform.position.y,
                            width: collider.width,
                            height: collider.height,
                        };
                        if rect.contains(x, y) {
                            entities.delete(entity);
                            build_resources.bits += 20;
                            break;
                        }
                    }
                }
                (MouseAction::BuildTower, MouseButton::Left) => {
                    // TODO: Move this to a system.
                    // If the player clicks on an open spot on the grid and has enough bits, then
                    // build a tower.
                    if  self.world.read_resource::<BuildResources>().bits >= 10 {
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

                            // Spend resources!
                            // TODO: Make the cost tunable in data somehow.
                            self.world.write_resource::<BuildResources>().bits -= 10;

                            println!("Built tower at {:?}!", (world_x, world_y));
                        };
                    }
                }
                (MouseAction::BuildTower, MouseButton::Right) => {
                    // If right clicking, cancel the tower building action.
                    self.mouse_action = MouseAction::Select;
                }
                _ => {}
            }
        }
    }

    fn key_down_event(&mut self, _ctx: &mut Context, keycode: KeyCode, _keymods: KeyMods, repeat: bool) {
        // Debug UI is taking keyboard input, so ignore this key.
        if self.debug_ui.io().want_capture_keyboard {
            return;
        }

        if repeat {
            return;
        }

        let play_state = *self.world.read_resource::<PlayState>();

        match keycode {
            KeyCode::R => {
                self.level_request = LoadLevelRequest::Reload;
            }
            KeyCode::Grave => {
                self.show_debug_ui = !self.show_debug_ui;
            }
            _ => {}
        }

        if play_state == PlayState::Play {
            match keycode {
                KeyCode::B if self.mouse_action == MouseAction::Select => {
                    self.mouse_action = MouseAction::BuildTower;
                }
                KeyCode::Escape if self.mouse_action == MouseAction::BuildTower => {
                    self.mouse_action = MouseAction::Select;
                }
                _ => {}
            }
        }
    }

    fn raw_winit_event(&mut self, ctx: &mut Context, event: &event::winit_event::Event) {
        // NOTE: This is called before any other event handlers.
        self.debug_ui.handle_event(ctx, event);
    }

    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        // TODO: Watch level directory for changes and update level_list if any files are
        // added/removed.

        // Call maintain to update all entities created via input events.
        self.world.maintain();

        let level_to_load = match &self.level_request {
            LoadLevelRequest::Reload => Some(&self.current_level),
            LoadLevelRequest::NewLevel(level_name) => Some(level_name),
            LoadLevelRequest::None => None,
        };

        if let Some(level_name) = level_to_load {
            level::load_level(level_name, &mut self.world);
            self.current_level = level_name.clone();
            self.level_request = LoadLevelRequest::None;
            self.mouse_action = MouseAction::Select;
        } else {
            let play_state = *self.world.read_resource::<PlayState>();

            if play_state != PlayState::Play {
                self.mouse_action = MouseAction::Select;
            }

            // Update world resources.
            {
                // Sets the time and updates it
                let duration = timer::duration_to_f64(timer::delta(ctx));
                self.world.insert(DeltaTime(duration as f32));

                // Clears collision event vector
                let mut collisions = self.world.write_resource::<Vec<CollisionEvent>>();
                collisions.clear();
                let mut death_events = self.world.write_resource::<Vec<DeathEvent>>();
                death_events.clear();
            }

            if play_state == PlayState::Play {
                self.dispatcher.dispatch(&mut self.world);

                // Update all entities created/deleted in systems.
                self.world.maintain();
            }
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        // Set up debug UI for this frame.
        self.debug_ui.prepare_frame(ctx)?;

        graphics::clear(ctx, graphics::BLACK);

        let system_data: (
            ReadStorage<Transform>,
            ReadStorage<Drawable>,
            ReadStorage<Shooter>,
            Read<BuildResources>,
            Read<Grid>,
            Read<PlayState>,
        ) = self.world.system_data();
        let (transforms, drawables, shooters, build_resources, grid, play_state) = system_data;

        let play_state = *play_state;

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

        // TODO: Sort drawables so enemies are rendered on top of buildings! Give them a Z order!

        // TODO: Instead of a Drawable, have Shape, Sprite, and other drawable Components.
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
                // TODO: Draw the Sun on top of the grid and the grid selection highlight.
                Drawable::Sun => {
                    graphics::Mesh::new_circle(
                        ctx,
                        graphics::DrawMode::fill(),
                        mint::Point2{x: 0.0, y: 0.0},
                        15.0,
                        0.1,
                        graphics::Color::from_rgb(252, 240, 3),
                    )?
                },
            };

            graphics::draw(ctx, &mesh, graphics::DrawParam::default().dest(transform.position))?;
        }

        // When building, highlight the grid cell the mouse is hovering over.
        if self.mouse_action == MouseAction::BuildTower {
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
        }

        // Draw shooter's attack radius.
        if input::keyboard::is_mod_active(ctx, KeyMods::ALT) {
            for (transform, shooter) in (&transforms, &shooters).join() {
                let mesh = graphics::Mesh::new_circle(
                    ctx,
                    graphics::DrawMode::fill(),
                    mint::Point2{x: 0.0, y: 0.0},
                    shooter.attack_radius,
                    0.1,
                    graphics::Color::from_rgba(60, 60, 60, 60),
                )?;
                graphics::draw(ctx, &mesh, graphics::DrawParam::default().dest(transform.position))?;
            }
        }

        // Draw amount of bits.
        // TODO: This ggez API to right-align text is dumb. I don't want to have to specify a
        // bounding size, just let me right-align the text.
        graphics::draw(
            ctx,
            graphics::Text::new(format!("Bits: {}", build_resources.bits))
                .set_bounds(Point2::new(400.0, f32::INFINITY), graphics::Align::Right),
            graphics::DrawParam::default()
                .dest([390.0, 10.0]),
        )?;

        match play_state {
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

        // Build and draw the debug UI.
        let level_request = &mut self.level_request;
        let level_list = &self.level_list;
        let current_level = &self.current_level;
        let mouse_action = &mut self.mouse_action;
        let show_debug_ui = self.show_debug_ui;
        self.debug_ui.draw_ui(ctx, |ui| {
            if show_debug_ui {
                ui.main_menu_bar(|| {
                    ui.menu(im_str!("Level")).build(|| {
                        ui.menu(im_str!("Load")).build(|| {
                            // Populate with list of levels.
                            for level_name in level_list {
                                let load_level = ui.menu_item(&im_str!("{}", level_name))
                                    .build();
                                if load_level {
                                    *level_request = LoadLevelRequest::NewLevel(level_name.clone());
                                }
                            }
                        });
                        let reload_level = ui.menu_item(&im_str!("Reload \"{}\"", current_level))
                            //.shortcut(im_str!("CTRL+R"))
                            .build();
                        if reload_level {
                            *level_request = LoadLevelRequest::Reload;
                        }
                    });
                });
            }

            //ui.show_demo_window(&mut true);

            if play_state == PlayState::Play {
                ui.window(im_str!("Towers"))
                    .position([500.0, 500.0], imgui::Condition::Always)
                    .size([300.0, 100.0], imgui::Condition::Always)
                    .resizable(false)
                    .movable(false)
                    .collapsible(false)
                    .build(|| {
                        let build_button_text = match mouse_action {
                            MouseAction::Select => im_str!("Build Tower (B)"),
                            MouseAction::BuildTower => im_str!("Cancel Build (Esc)"),
                        };
                        if ui.button(build_button_text, [120.0, 60.0]) {
                            *mouse_action = match mouse_action {
                                MouseAction::Select => MouseAction::BuildTower,
                                MouseAction::BuildTower => MouseAction::Select,
                            };
                        }
                    });
            }
        });

        // NOTE: Add any over-UI rendering here.

        graphics::present(ctx)?;
        Ok(())
    }
}

impl<'a, 'b> State<'a, 'b> {
    fn new(ctx: &mut Context) -> GameResult<Self> {
        // Set up the specs world.
        let mut world = World::new();
        world.register::<Transform>();
        world.register::<Drawable>();
        // Currently the Projectile resource isn't accessed by any systems so it needs to be registered here.
        world.register::<Projectile>();

        let mut dispatcher = DispatcherBuilder::new()
            .with(EnemyAi, "enemy_ai", &[])
            .with(modes::pvz::systems::SunBehaviorSystem, "sun_behavior", &["enemy_ai"])
            .with(ShooterSystem, "shooter_system", &["sun_behavior"])
            .with(UpdatePosition, "update_position", &["shooter_system"])
            .with(CollisionSystem, "collision_system", &["update_position"])
            .with(AttackSystem, "attack_system", &["collision_system"])
            .with(SpawnerSystem, "spawner_system", &["attack_system"])
            .with(modes::pvz::systems::SunSpawnSystem::new(), "sun_spawn", &["spawner_system"])
            .with(DeathSystem, "death_system", &["sun_spawn"])
            .with(WinSystem, "win_system", &["death_system"])
            .build();

        dispatcher.setup(&mut world);

        // Load the level!
        let start_level = "test";
        level::load_level(start_level, &mut world);

        // Initialize the debug UI.
        let debug_ui = DebugUi::new(ctx);

        let level_list = level::find_levels();

        Ok(Self {
            world,
            dispatcher,
            current_level: start_level.to_owned(),
            level_request: LoadLevelRequest::None,

            mouse_action: MouseAction::Select,

            debug_ui,
            show_debug_ui: false,
            draw_colliders: false,
            level_list,
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
