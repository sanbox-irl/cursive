//! Various views to use when creating the layout.

/// A macro to help with creating toggleable views.
///
/// # Examples
///
/// ```
/// struct MyView {
///     enabled: bool,
/// }
///
/// impl MyView {
///     cursive::impl_enabled!(self.enabled);
/// }
///
/// let view = MyView { enabled: true };
/// assert!(view.is_enabled());
/// ```
#[macro_export]
macro_rules! impl_enabled {
    (self.$x:ident) => {

        /// Disables this view.
        ///
        /// A disabled view cannot be selected.
        pub fn disable(&mut self) {
            self.$x = false;
        }

        /// Disables this view.
        ///
        /// Chainable variant.
        pub fn disabled(self) -> Self {
            use $crate::traits::With as _;
            self.with(Self::disable)
        }

        /// Re-enables this view.
        pub fn enable(&mut self) {
            self.$x = true;
        }

        /// Enable or disable this view.
        pub fn set_enabled(&mut self, enabled: bool) {
            self.$x = enabled;
        }

        /// Returns `true` if this view is enabled.
        pub fn is_enabled(&self) -> bool {
            self.$x
        }
    }
}

mod boxed;
mod button;
mod canvas;
mod checkbox;
mod circular_focus;
mod debug_console;
mod dialog;
mod dummy;
mod edit;
mod enableable;
mod hideable;
mod last_size;
mod layer;
mod linear_layout;
mod list;
mod menu_popup;
mod menubar;
mod named;
mod on_event;
mod padded;
mod panel;
mod progress_bar;
mod radio;
mod resized;
mod scroll;
mod select;
mod shadow;
mod slider;
mod stack;
mod text;
mod text_area;
mod tracked;

pub use self::boxed::Boxed;
pub use self::button::Button;
pub use self::canvas::Canvas;
pub use self::checkbox::Checkbox;
pub use self::circular_focus::CircularFocus;
pub use self::debug_console::DebugConsole;
pub use self::dialog::{Dialog, DialogFocus};
pub use self::dummy::Dummy;
pub use self::edit::Edit;
pub use self::enableable::Enableable;
pub use self::hideable::Hideable;
pub use self::last_size::LastSize;
pub use self::layer::Layer;
pub use self::linear_layout::LinearLayout;
pub use self::list::{List, ListChild};
pub use self::menu_popup::MenuPopup;
pub use self::menubar::Menubar;
pub use self::named::{Named, ViewRef};
pub use self::on_event::OnEvent;
pub use self::padded::Padded;
pub use self::panel::Panel;
pub use self::progress_bar::ProgressBar;
pub use self::radio::{RadioButton, RadioGroup};
pub use self::resized::Resized;
pub use self::scroll::Scroll;
pub use self::select::Select;
pub use self::shadow::Shadow;
pub use self::slider::Slider;
pub use self::stack::{LayerPosition, Stack};
pub use self::text::{Text, TextContent, TextContentRef};
pub use self::text_area::TextArea;
pub use self::tracked::Tracked;
