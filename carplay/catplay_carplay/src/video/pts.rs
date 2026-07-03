use std::{
    fmt,
    time::{Duration, Instant},
};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Pts(pub Instant);

pub enum PtsDecoded {
    Past(Duration),
    Future(Duration),
}

impl PtsDecoded {
    pub fn is_past(&self) -> bool {
        matches!(self, PtsDecoded::Past(_))
    }

    pub fn delta(&self) -> Duration {
        match self {
            PtsDecoded::Past(dur) => *dur,
            PtsDecoded::Future(dur) => *dur,
        }
    }

    pub fn past(&self) -> Duration {
        match self {
            PtsDecoded::Past(dur) => *dur,
            PtsDecoded::Future(_) => Duration::ZERO,
        }
    }

    pub fn future(&self) -> Duration {
        match self {
            PtsDecoded::Past(_) => Duration::ZERO,
            PtsDecoded::Future(dur) => *dur,
        }
    }
}

impl Pts {
    pub fn is_past(&self) -> bool {
        self.decode().is_past()
    }

    pub fn decode(&self) -> PtsDecoded {
        let now = Instant::now();
        let pts_past = if now > self.0 { now - self.0 } else { Duration::ZERO };
        let pts_future = if self.0 > now { self.0 - now } else { Duration::ZERO };
        match pts_past.is_zero() {
            false => PtsDecoded::Past(pts_past),
            true => PtsDecoded::Future(pts_future),
        }
    }
}

impl From<Instant> for Pts {
    fn from(value: Instant) -> Self {
        Self(value)
    }
}

impl fmt::Debug for PtsDecoded {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PtsDecoded::Past(dur) => f.write_fmt(format_args!("Pts(-{dur:?})")),
            PtsDecoded::Future(dur) => f.write_fmt(format_args!("Pts({dur:?})")),
        }
    }
}

impl fmt::Display for PtsDecoded {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PtsDecoded::Past(duration) => f.write_fmt(format_args!("-{duration:?}")),
            PtsDecoded::Future(duration) => f.write_fmt(format_args!("{duration:?}")),
        }
    }
}

impl fmt::Debug for Pts {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.decode().fmt(f)
    }
}

impl fmt::Display for Pts {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.decode().fmt(f)
    }
}
