extern crate alloc;

use alloc::sync::Arc;
use core::{fmt, ops::Deref};

/// `Arc<T>` but with `PartialEq` that uses `Arc::ptr_eq`.
pub struct ArcBox<T: ?Sized>(Arc<T>);

impl<T> ArcBox<T> {
    pub fn new(inner: T) -> Self {
        ArcBox(Arc::new(inner))
    }

    pub fn into_inner(self) -> Arc<T> {
        self.0
    }
}

impl<T> From<T> for ArcBox<T> {
    fn from(inner: T) -> Self {
        ArcBox::new(inner)
    }
}

impl<T: ?Sized> ArcBox<T> {
    pub fn from_box(inner: Box<T>) -> Self {
        ArcBox(Arc::from(inner))
    }

    pub fn from_arc(inner: Arc<T>) -> Self {
        ArcBox(inner)
    }
}

impl<T: ?Sized> From<Arc<T>> for ArcBox<T> {
    fn from(inner: Arc<T>) -> Self {
        ArcBox::from_arc(inner)
    }
}

impl<T: ?Sized> From<Box<T>> for ArcBox<T> {
    fn from(inner: Box<T>) -> Self {
        ArcBox::from_box(inner)
    }
}

impl<T: ?Sized> Clone for ArcBox<T> {
    fn clone(&self) -> Self {
        ArcBox(Arc::clone(&self.0))
    }
}

impl<T: ?Sized> PartialEq for ArcBox<T> {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

// impl<T: ?Sized> Eq for ArcBox<T> {}

impl<T: ?Sized> Deref for ArcBox<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: fmt::Debug + ?Sized> fmt::Debug for ArcBox<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<T: fmt::Display + ?Sized> fmt::Display for ArcBox<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
