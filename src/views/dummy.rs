use crate::view::View;
use crate::Printer;

/// Dummy view.
///
/// Doesn't print anything. Minimal size is (1,1).
pub struct Dummy;

impl View for Dummy {
    fn draw(&self, _: &Printer<'_, '_>) {}

    fn needs_relayout(&self) -> bool {
        false
    }
}
