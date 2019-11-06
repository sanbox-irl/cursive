use crate::event::{Event, EventResult};
use crate::view::ViewWrapper;
use crate::{Printer, View};

/// Wrapper around another view that can be enabled/disabled at will.
///
/// When disabled, all child views will be disabled and will stop receiving events.
///
/// # Examples
///
/// ```
/// use cursive::Cursive;
/// use cursive::views::{Button, Enableable, Checkbox, LinearLayout};
/// use cursive::traits::Identifiable;
///
/// let mut siv = Cursive::dummy();
///
/// siv.add_layer(LinearLayout::vertical()
///     .child(Enableable::new(Checkbox::new()).with_id("my_view"))
///     .child(Button::new("Toggle", |s| {
///         s.call_on_id("my_view", |v: &mut Enableable<Checkbox>| {
///             // This will disable (or re-enable) the checkbox, preventing the user from
///             // interacting with it.
///             v.set_enabled(!v.is_enabled());
///         });
///     }))
/// );
/// ```
pub struct Enableable<V> {
    view: V,
    enabled: bool,
}

impl<V> Enableable<V> {
    /// Creates a new `Enableable` around `view`.
    ///
    /// It will be enabled by default.
    pub fn new(view: V) -> Self {
        Enableable {
            view,
            enabled: true,
        }
    }

    impl_enabled!(self.enabled);
    inner_getters!(self.view: V);
}

impl<V: View> ViewWrapper for Enableable<V> {
    wrap_impl!(self.view: V);

    fn wrap_on_event(&mut self, event: Event) -> EventResult {
        if self.enabled {
            self.view.on_event(event)
        } else {
            EventResult::Ignored
        }
    }

    fn wrap_draw(&self, printer: &Printer<'_, '_>) {
        self.view.draw(&printer.enabled(self.enabled));
    }
}
