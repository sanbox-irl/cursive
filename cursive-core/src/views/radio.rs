use crate::direction::Direction;
use crate::event::{Event, EventResult, Key, MouseButton, MouseEvent};
use crate::theme::ColorStyle;
use crate::view::View;
use crate::Cursive;
use crate::Vec2;
use crate::{Printer, With};
use std::cell::RefCell;
use std::rc::Rc;

struct SharedState<T> {
    selection: usize,
    values: Vec<Rc<T>>,

    on_change: Option<Rc<dyn Fn(&mut Cursive, &T)>>,
}

impl<T> SharedState<T> {
    pub fn selection(&self) -> Rc<T> {
        Rc::clone(&self.values[self.selection])
    }
}

/// Group to coordinate multiple radio buttons.
///
/// A `RadioGroup` is used to create and manage [`RadioButton`]s.
///
/// A `RadioGroup` can be cloned; it will keep pointing to the same group.
#[derive(Clone)]
pub struct RadioGroup<T> {
    // Given to every child button
    state: Rc<RefCell<SharedState<T>>>,
    config: RadioButtonConfig,
}

impl<T: 'static> Default for RadioGroup<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: 'static> RadioGroup<T> {
    /// Creates an empty group for radio buttons. It will use the default
    /// [`RadioButtonConfig`] [`DEFAULT_RADIO_BUTTON_CONFIG`].
    pub fn new() -> Self {
        RadioGroup {
            state: Rc::new(RefCell::new(SharedState {
                selection: 0,
                values: Vec::new(),
                on_change: None,
            })),
            config: DEFAULT_RADIO_BUTTON_CONFIG,
        }
    }

    /// Creates an empty group for radio buttons with a [`RadioButtonConfig`].
    pub fn with_config(config: RadioButtonConfig) -> Self {
        RadioGroup {
            state: Rc::new(RefCell::new(SharedState {
                selection: 0,
                values: Vec::new(),
                on_change: None,
            })),
            config,
        }
    }

    /// Adds a new button to the group.
    ///
    /// The button will display `label` next to it, and will embed `value`.
    pub fn button<S: Into<String>>(
        &mut self,
        value: T,
        label: S,
    ) -> RadioButton<T> {
        let count = self.state.borrow().values.len();
        self.state.borrow_mut().values.push(Rc::new(value));
        RadioButton::new(
            Rc::clone(&self.state),
            count,
            label.into(),
            self.config,
        )
    }

    /// Returns the id of the selected button.
    ///
    /// Buttons are indexed in the order they are created, starting from 0.
    pub fn selected_id(&self) -> usize {
        self.state.borrow().selection
    }

    /// Returns the value associated with the selected button.
    pub fn selection(&self) -> Rc<T> {
        self.state.borrow().selection()
    }

    /// Sets a callback to be used when the selection changes.
    pub fn set_on_change<F: 'static + Fn(&mut Cursive, &T)>(
        &mut self,
        on_change: F,
    ) {
        self.state.borrow_mut().on_change = Some(Rc::new(on_change));
    }

    /// Sets a callback to be used when the selection changes.
    ///
    /// Chainable variant.
    pub fn on_change<F: 'static + Fn(&mut Cursive, &T)>(
        self,
        on_change: F,
    ) -> Self {
        self.with(|s| s.set_on_change(on_change))
    }
}

impl RadioGroup<String> {
    /// Adds a button, using the label itself as value.
    pub fn button_str<S: Into<String>>(
        &mut self,
        text: S,
    ) -> RadioButton<String> {
        let text = text.into();
        self.button(text.clone(), text)
    }
}

/// Variant of `Checkbox` arranged in group.
///
/// `RadioButton`s are managed by a [`RadioGroup`]. A single group can contain
/// several radio buttons, but only one button per group can be active at a
/// time.
///
/// `RadioButton`s are not created directly, but through
/// [`RadioGroup::button`].
///
pub struct RadioButton<T> {
    state: Rc<RefCell<SharedState<T>>>,
    id: usize,
    enabled: bool,
    label: String,
    config: RadioButtonConfig,
}

impl<T: 'static> RadioButton<T> {
    impl_enabled!(self.enabled);

    fn new(
        state: Rc<RefCell<SharedState<T>>>,
        id: usize,
        label: String,
        config: RadioButtonConfig,
    ) -> Self {
        RadioButton {
            state,
            id,
            enabled: true,
            label,
            config,
        }
    }

    /// Returns `true` if this button is selected.
    pub fn is_selected(&self) -> bool {
        self.state.borrow().selection == self.id
    }

    /// Selects this button, un-selecting any other in the same group.
    pub fn select(&mut self) -> EventResult {
        let mut state = self.state.borrow_mut();
        state.selection = self.id;
        if let Some(ref on_change) = state.on_change {
            let on_change = Rc::clone(on_change);
            let value = state.selection();
            EventResult::with_cb(move |s| on_change(s, &value))
        } else {
            EventResult::Consumed(None)
        }
    }

    /// Selects this button, un-selecting any other in the same group.
    ///
    /// Chainable variant.
    pub fn selected(self) -> Self {
        self.with(|s| {
            // Ignore the potential callback here
            s.select();
        })
    }

    /// Sets the selection status of the button.
    ///
    /// Chainable variant.
    pub fn with_selected(self, selection: bool) -> Self {
        self.with(|s| {
            // Ignore the potential callback here
            if selection {
                s.select();
            }
        })
    }

    fn draw_internal(&self, printer: &Printer<'_, '_>) {
        let mut printer_xpos = 0;
        printer.print((printer_xpos, 0), self.config.left_bracket);
        printer_xpos += self.config.left_bracket.len();

        if self.is_selected() {
            printer.print((printer_xpos, 0), self.config.check);
            printer_xpos += self.config.check.len();
        } else {
            printer.print((printer_xpos, 0), self.config.uncheck);
            printer_xpos += self.config.uncheck.len();
        }

        printer.print((printer_xpos, 0), self.config.right_bracket);
        printer_xpos += self.config.right_bracket.len();

        if !self.label.is_empty() {
            // We want the space to be highlighted if focused
            printer.print((printer_xpos, 0), self.config.post_label_space);
            printer_xpos += self.config.post_label_space.len();

            printer.print((printer_xpos, 0), &self.label);
        }
    }

    fn req_size(&self) -> Vec2 {
        let base_length = self.config.left_bracket.len()
            + if self.is_selected() {
                self.config.check.len()
            } else {
                self.config.uncheck.len()
            }
            + self.config.right_bracket.len();

        if self.label.is_empty() {
            Vec2::new(base_length, 1)
        } else {
            Vec2::new(
                base_length
                    + self.config.post_label_space.len()
                    + self.label.len(),
                1,
            )
        }
    }
}

impl<T: 'static> View for RadioButton<T> {
    fn required_size(&mut self, _: Vec2) -> Vec2 {
        self.req_size()
    }

    fn take_focus(&mut self, _: Direction) -> bool {
        self.enabled
    }

    fn draw(&self, printer: &Printer<'_, '_>) {
        if self.enabled && printer.enabled {
            printer.with_selection(printer.focused, |printer| {
                self.draw_internal(printer)
            });
        } else {
            printer.with_color(ColorStyle::secondary(), |printer| {
                self.draw_internal(printer)
            });
        }
    }

    fn on_event(&mut self, event: Event) -> EventResult {
        match event {
            Event::Key(Key::Enter) | Event::Char(' ') => self.select(),
            Event::Mouse {
                event: MouseEvent::Release(MouseButton::Left),
                position,
                offset,
            } if position.fits_in_rect(offset, self.req_size()) => {
                self.select()
            }
            _ => EventResult::Ignored,
        }
    }
}

/// This is the default radio button configuration,
/// used with the [`RadioGroup::button`] method.
pub const DEFAULT_RADIO_BUTTON_CONFIG: RadioButtonConfig = RadioButtonConfig {
    check: "X",
    uncheck: " ",
    left_bracket: "(",
    right_bracket: ")",
    post_label_space: " ",
};

/// The Configuration of a Radio Button, setting what a "check",
/// "uncheck", and surrounding bracket pair look like. Although all
/// the default options are single characters, a user can create
/// multi-character marks as well.
#[derive(Debug, Clone, Copy)]
pub struct RadioButtonConfig {
    /// A mark for when a RadioButton is selected.
    ///
    /// Defaults to `X`.
    pub check: &'static str,

    /// A mark for when a RadioButton is *not* selected.
    ///
    /// Default to ` `.
    pub uncheck: &'static str,

    /// A mark for the Left Bracket of a RadioButton.
    ///
    /// It defaults to `(`.
    pub left_bracket: &'static str,

    /// A mark for the Right Bracket of a RadioButton.
    ///
    /// It defaults to `)`.
    pub right_bracket: &'static str,

    /// A mark for the space between the right bracket and the
    /// label. For example, if a RadioButton looked like:
    ///
    /// ```
    /// (X) Selected
    /// ```
    /// the `post_label_space` is the space character between `)` and
    /// `S`.
    ///
    /// It defaults to ` `.
    pub post_label_space: &'static str,
}
