#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DirtyRect {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

impl DirtyRect {
    pub fn union(a: Self, b: Self) -> Self {
        let left = a.x.min(b.x);
        let top = a.y.min(b.y);
        let right = (a.x + a.w).max(b.x + b.w);
        let bottom = (a.y + a.h).max(b.y + b.h);

        Self {
            x: left,
            y: top,
            w: right - left,
            h: bottom - top,
        }
    }

    pub fn merge(&mut self, other: Self) {
        *self = Self::union(*self, other)
    }
}
