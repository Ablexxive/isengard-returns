/// A rectangle centered at (x, y).
#[derive(Clone, Default)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    #[inline]
    pub fn left(&self) -> f32 {
        self.x - (self.width as f32 / 2.0)
    }

    #[inline]
    pub fn right(&self) -> f32 {
        self.x + (self.width as f32 / 2.0)
    }

    #[inline]
    pub fn top(&self) -> f32 {
        self.y - (self.height as f32 / 2.0)
    }

    #[inline]
    pub fn bottom(&self) -> f32 {
        self.y + (self.height as f32 / 2.0)
    }

    pub fn overlaps(&self, other: &Rect) -> bool {
        self.right() > other.left() &&
        self.left() < other.right() &&
        self.top() < other.bottom() &&
        self.bottom() > other.top()
    }
}
