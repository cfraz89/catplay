use crate::DirtyRect;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Unit {
    Px(f32),
    Dp(f32),
    Percent(f32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Anchor {
    TopLeft,
    TopCenter,
    TopRight,

    CenterLeft,
    Center,
    CenterRight,

    BottomLeft,
    BottomCenter,
    BottomRight,

    // For text
    BaselineLeft,
    BaselineCenter,
    BaselineRight,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextAlign {
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UiRect {
    pub x: Unit,
    pub y: Unit,
    pub w: Unit,
    pub h: Unit,
    pub anchor: Anchor,
}

pub fn resolve_unit(u: Unit, total: i32, dpi: f32) -> u32 {
    match u {
        Unit::Px(v) => v as u32,
        Unit::Dp(v) => (v * dpi / 160.0) as u32,
        Unit::Percent(p) => (p * total as f32) as u32,
    }
}

pub fn resolve_rect(r: UiRect, w: i32, h: i32, dpi: f32) -> DirtyRect {
    let rw = resolve_unit(r.w, w, dpi);
    let rh = resolve_unit(r.h, h, dpi);

    let mut x = resolve_unit(r.x, w, dpi);
    let mut y = resolve_unit(r.y, h, dpi);

    match r.anchor {
        Anchor::TopLeft => {}
        Anchor::TopCenter => {
            x -= rw / 2;
        }
        Anchor::TopRight => {
            x -= rw;
        }

        Anchor::CenterLeft => {
            y -= rh / 2;
        }
        Anchor::Center => {
            x -= rw / 2;
            y -= rh / 2;
        }
        Anchor::CenterRight => {
            x -= rw;
            y -= rh / 2;
        }

        Anchor::BottomLeft => {
            y -= rh;
        }
        Anchor::BottomCenter => {
            x -= rw / 2;
            y -= rh;
        }
        Anchor::BottomRight => {
            x -= rw;
            y -= rh;
        }

        Anchor::BaselineLeft => {}
        Anchor::BaselineCenter => {
            x -= rw / 2;
        }
        Anchor::BaselineRight => {
            x -= rw;
        }
    }

    DirtyRect { x, y, w: rw, h: rh }
}
