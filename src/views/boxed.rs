use crate::view::{IntoBoxedView, View, ViewWrapper};
use std::ops::{Deref, DerefMut};

/// A boxed `View`.
///
/// It derefs to the wrapped view.
pub struct Boxed {
    view: Box<dyn View>,
}

impl Boxed {
    /// Creates a new `Boxed` around the given boxed view.
    pub fn new(view: Box<dyn View>) -> Self {
        Boxed { view }
    }

    /// Box the given view
    pub fn boxed<T>(view: T) -> Self
    where
        T: IntoBoxedView,
    {
        Boxed::new(view.as_boxed_view())
    }

    /// Returns the inner boxed view.
    pub fn unwrap(self) -> Box<dyn View> {
        self.view
    }
}

impl Deref for Boxed {
    type Target = dyn View;

    fn deref(&self) -> &dyn View {
        &*self.view
    }
}

impl DerefMut for Boxed {
    fn deref_mut(&mut self) -> &mut dyn View {
        &mut *self.view
    }
}

impl ViewWrapper for Boxed {
    type V = dyn View;

    fn with_view<F, R>(&self, f: F) -> Option<R>
    where
        F: FnOnce(&Self::V) -> R,
    {
        Some(f(&*self.view))
    }

    fn with_view_mut<F, R>(&mut self, f: F) -> Option<R>
    where
        F: FnOnce(&mut Self::V) -> R,
    {
        Some(f(&mut *self.view))
    }
}
