/// A rectangle centered at (x, y).
#[derive(Clone, Default)]
pub struct Rectangle {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rectangle {
    pub fn new() -> Rectangle {
        Rectangle {
            x: 0.0,
            y: 0.0,
            width: 0,
            height: 0,
        }
    }

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

    pub fn overlaps(&self, other: &Rectangle) -> bool {
        self.right() > other.left() &&
        self.left() < other.right() &&
        self.top() < other.bottom() &&
        self.bottom() > other.top()
    }
}
