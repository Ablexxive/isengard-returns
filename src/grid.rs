#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GridCell {
    Walkable,
    Buildable,
    Occupied,
}

#[derive(Clone, Debug, Default)]
pub struct Grid {
    pub width: u32,
    pub height: u32,
    pub cell_size: f32,
    pub grid: Vec<GridCell>,
}

impl Grid {
    pub fn new(width: u32, height: u32, cell_size: f32) -> Self {
        Self {
            width,
            height,
            cell_size,
            grid: vec![GridCell::Buildable; (width * height) as usize],
        }
    }

    pub fn get_cell(&self, x: u32, y: u32) -> Option<GridCell> {
        self.grid.get((y * self.width + x) as usize).cloned()
    }

    pub fn set_cell(&mut self, x: u32, y: u32, value: GridCell) -> bool {
        if let Some(cell) = self.grid.get_mut((y * self.width + x) as usize) {
            *cell = value;
            true
        } else {
            false
        }
    }

    pub fn is_walkable(&self, x: u32, y: u32) -> bool {
        self.get_cell(x, y) == Some(GridCell::Walkable)
    }

    pub fn is_buildable(&self, x: u32, y: u32) -> bool {
        self.get_cell(x, y) == Some(GridCell::Buildable)
    }

    pub fn is_occupied(&self, x: u32, y: u32) -> bool {
        self.get_cell(x, y) == Some(GridCell::Occupied)
    }
}
