use crate::view::View;
use crate::views::Scroll;

/// Makes a view wrappable in a [`Scroll`].
///
/// [`Scroll`]: crate::views::Scroll
pub trait Scrollable: View + Sized {
    /// Wraps `self` in a `Scroll`.
    fn scrollable(self) -> Scroll<Self> {
        Scroll::new(self)
    }
}

impl<T: View> Scrollable for T {}
