use crate::vec::Vec2;
use crate::view::{View, ViewWrapper};
use crate::views::Named;
use crate::Printer;
use std::cell::Cell;

/// Wrapper around a view that remembers its position.
pub struct Tracked<T: View> {
    /// Wrapped view.
    pub view: T,
    /// Last position the view was located.
    offset: Cell<Vec2>,
}

impl<T: View> Tracked<T> {
    /// Return the last offset at which the view was drawn.
    pub fn offset(&self) -> Vec2 {
        self.offset.get()
    }

    /// Creates a new `Tracked` around `view`.
    pub fn new(view: T) -> Self {
        Tracked {
            view,
            offset: Cell::new(Vec2::zero()),
        }
    }

    /// Wraps itself in a `Named` for easy retrieval.
    pub fn with_id(self, id: &str) -> Named<Self> {
        Named::new(id, self)
    }

    inner_getters!(self.view: T);
}

impl<T: View> ViewWrapper for Tracked<T> {
    wrap_impl!(self.view: T);

    fn wrap_draw(&self, printer: &Printer<'_, '_>) {
        self.offset.set(printer.offset);
        self.view.draw(printer);
    }
}
