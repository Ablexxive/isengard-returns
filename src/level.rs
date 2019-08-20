use tiled::PropertyValue;
use specs::prelude::*;

use crate::grid::*;
use crate::components::*;
use crate::resources::*;

pub fn load_level(world: &mut World) {
    // Clear out world first and reset resources.
    world.delete_all();
    world.insert(PlayState::Play);

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
}
