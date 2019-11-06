use crate::vec::Vec2;
use crate::view::{SizeConstraint, View};
use crate::views::Resized;

/// Makes a view wrappable in a [`Resized`].
///
/// [`Resized`]: ../views/struct.Resized.html
pub trait Boxable: View + Sized {
    /// Wraps `self` in a `Resized` with the given size constraints.
    fn boxed(
        self,
        width: SizeConstraint,
        height: SizeConstraint,
    ) -> Resized<Self> {
        Resized::new(width, height, self)
    }

    /// Wraps `self` into a fixed-size `Resized`.
    fn fixed_size<S: Into<Vec2>>(self, size: S) -> Resized<Self> {
        Resized::with_fixed_size(size, self)
    }

    /// Wraps `self` into a fixed-width `Resized`.
    fn fixed_width(self, width: usize) -> Resized<Self> {
        Resized::with_fixed_width(width, self)
    }

    /// Wraps `self` into a fixed-width `Resized`.
    fn fixed_height(self, height: usize) -> Resized<Self> {
        Resized::with_fixed_height(height, self)
    }

    /// Wraps `self` into a full-screen `Resized`.
    fn full_screen(self) -> Resized<Self> {
        Resized::with_full_screen(self)
    }

    /// Wraps `self` into a full-width `Resized`.
    fn full_width(self) -> Resized<Self> {
        Resized::with_full_width(self)
    }

    /// Wraps `self` into a full-height `Resized`.
    fn full_height(self) -> Resized<Self> {
        Resized::with_full_height(self)
    }

    /// Wraps `self` into a limited-size `Resized`.
    fn max_size<S: Into<Vec2>>(self, size: S) -> Resized<Self> {
        Resized::with_max_size(size, self)
    }

    /// Wraps `self` into a limited-width `Resized`.
    fn max_width(self, max_width: usize) -> Resized<Self> {
        Resized::with_max_width(max_width, self)
    }

    /// Wraps `self` into a limited-height `Resized`.
    fn max_height(self, max_height: usize) -> Resized<Self> {
        Resized::with_max_height(max_height, self)
    }

    /// Wraps `self` into a `Resized` at least sized `size`.
    fn min_size<S: Into<Vec2>>(self, size: S) -> Resized<Self> {
        Resized::with_min_size(size, self)
    }

    /// Wraps `self` in a `Resized` at least `min_width` wide.
    fn min_width(self, min_width: usize) -> Resized<Self> {
        Resized::with_min_width(min_width, self)
    }

    /// Wraps `self` in a `Resized` at least `min_height` tall.
    fn min_height(self, min_height: usize) -> Resized<Self> {
        Resized::with_min_height(min_height, self)
    }
}

impl<T: View> Boxable for T {}
