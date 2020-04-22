use std::any::Any;
use std::num::NonZeroU32;
#[cfg(feature = "toml")]
use std::path::Path;
use std::time::Duration;

use crossbeam_channel::{self, Receiver, Sender};

use crate::backend;
use crate::direction;
use crate::event::{Event, EventResult};
use crate::printer::Printer;
use crate::theme;
use crate::view::{self, Finder, IntoBoxedView, Position, View};
use crate::views::{self, LayerPosition};
use crate::Vec2;

static DEBUG_VIEW_NAME: &str = "_cursive_debug_view";

// How long we wait between two empty input polls
const INPUT_POLL_DELAY_MS: u64 = 30;

/// Central part of the cursive library.
///
/// It initializes ncurses on creation and cleans up on drop.
/// To use it, you should populate it with views, layouts, and callbacks,
/// then start the event loop with `run()`.
///
/// It uses a list of screen, with one screen active at a time.
pub struct Cursive {
    theme: theme::Theme,

    // The main view
    root: views::OnEventView<views::ScreensView<views::StackView>>,

    menubar: views::Menubar,

    // Last layer sizes of the stack view.
    // If it changed, clear the screen.
    last_sizes: Vec<Vec2>,

    running: bool,

    backend: Box<dyn backend::Backend>,

    // Handle asynchronous callbacks
    cb_source: Receiver<Box<dyn FnOnce(&mut Cursive) + Send>>,
    cb_sink: Sender<Box<dyn FnOnce(&mut Cursive) + Send>>,

    // User-provided data.
    user_data: Box<dyn Any>,

    // Handle auto-refresh when no event is received.
    fps: Option<NonZeroU32>,
    boring_frame_count: u32,
}

/// Identifies a screen in the cursive root.
pub type ScreenId = usize;

/// Convenient alias to the result of `Cursive::cb_sink`.
///
/// # Notes
///
/// Callbacks need to be `Send`, which can be limiting in some cases.
///
/// In some case [`send_wrapper`] may help you work around that.
///
/// [`send_wrapper`]: https://crates.io/crates/send_wrapper
pub type CbSink = Sender<Box<dyn FnOnce(&mut Cursive) + Send>>;

impl Cursive {
    /// Shortcut for `Cursive::try_new` with non-failible init function.
    ///
    /// You probably don't want to use this function directly, unless you're
    /// using a non-standard backend. Built-in backends have dedicated functions.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use cursive_core::{Cursive, backend};
    /// let siv = Cursive::new(backend::Dummy::init);
    /// ```
    pub fn new<F>(backend_init: F) -> Self
    where
        F: FnOnce() -> Box<dyn backend::Backend>,
    {
        Self::try_new::<_, ()>(|| Ok(backend_init())).unwrap()
    }

    /// Creates a new Cursive root, and initialize the back-end.
    ///
    /// You probably don't want to use this function directly, unless you're
    /// using a non-standard backend. Built-in backends have dedicated functions in the
    /// [`CursiveExt`] trait.
    ///
    /// [`CursiveExt`]: https://docs.rs/cursive/0/cursive/trait.CursiveExt.html
    pub fn try_new<F, E>(backend_init: F) -> Result<Self, E>
    where
        F: FnOnce() -> Result<Box<dyn backend::Backend>, E>,
    {
        let theme = theme::load_default();

        let (cb_sink, cb_source) = crossbeam_channel::unbounded();

        let backend = backend_init()?;
        let mut cursive = Cursive {
            theme,
            root: views::OnEventView::new(views::ScreensView::single_screen(
                views::StackView::new(),
            )),
            last_sizes: Vec::new(),
            menubar: views::Menubar::new(),
            running: true,
            cb_source,
            cb_sink,
            backend,
            fps: None,
            boring_frame_count: 0,
            user_data: Box::new(()),
        };
        cursive.reset_default_callbacks();

        Ok(cursive)
    }

    /*
    /// Creates a new Cursive root using a ncurses backend.
    #[cfg(feature = "ncurses-backend")]
    pub fn ncurses() -> std::io::Result<Self> {
        Self::try_new(backend::curses::n::Backend::init)
    }

    /// Creates a new Cursive root using a pancurses backend.
    #[cfg(feature = "pancurses-backend")]
    pub fn pancurses() -> std::io::Result<Self> {
        Self::try_new(backend::curses::pan::Backend::init)
    }

    /// Creates a new Cursive root using a termion backend.
    #[cfg(feature = "termion-backend")]
    pub fn termion() -> std::io::Result<Self> {
        Self::try_new(backend::termion::Backend::init)
    }

    /// Creates a new Cursive root using a crossterm backend.
    #[cfg(feature = "crossterm-backend")]
    pub fn crossterm() -> Result<Self, crossterm::ErrorKind> {
        Self::try_new(backend::crossterm::Backend::init)
    }

    /// Creates a new Cursive root using a bear-lib-terminal backend.
    #[cfg(feature = "blt-backend")]
    pub fn blt() -> Self {
        Self::new(backend::blt::Backend::init)
    }
    */

    /// Creates a new Cursive root using a [dummy backend].
    ///
    /// Nothing will be output. This is mostly here for tests.
    ///
    /// [dummy backend]: backend::Dummy
    pub fn dummy() -> Self {
        Self::new(backend::Dummy::init)
    }

    /// Sets some data to be stored in Cursive.
    ///
    /// It can later on be accessed with `Cursive::user_data()`
    pub fn set_user_data<T: Any>(&mut self, user_data: T) {
        self.user_data = Box::new(user_data);
    }

    /// Attempts to access the user-provided data.
    ///
    /// If some data was set previously with the same type, returns a
    /// reference to it.
    ///
    /// If nothing was set or if the type is different, returns `None`.
    pub fn user_data<T: Any>(&mut self) -> Option<&mut T> {
        self.user_data.downcast_mut()
    }

    /// Attemps to take by value the current user-data.
    ///
    /// If successful, this will replace the current user-data with the unit
    /// type `()`.
    ///
    /// If the current user data is not of the requested type, `None` will be
    /// returned.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let mut siv = cursive_core::Cursive::dummy();
    ///
    /// // Start with a simple `Vec<i32>` as user data.
    /// siv.set_user_data(vec![1i32, 2, 3]);
    /// assert_eq!(siv.user_data::<Vec<i32>>(), Some(&mut vec![1i32, 2, 3]));
    ///
    /// // Let's mutate the data a bit.
    /// siv.with_user_data(|numbers: &mut Vec<i32>| numbers.push(4));
    ///
    /// // If mutable reference is not enough, we can take the data by value.
    /// let data: Vec<i32> = siv.take_user_data().unwrap();
    /// assert_eq!(data, vec![1i32, 2, 3, 4]);
    ///
    /// // At this point the user data was removed and is no longer available.
    /// assert_eq!(siv.user_data::<Vec<i32>>(), None);
    /// ```
    pub fn take_user_data<T: Any>(&mut self) -> Option<T> {
        // Start by taking the user data and replacing it with a dummy.
        let user_data = std::mem::replace(&mut self.user_data, Box::new(()));

        // Downcast the data to the requested type.
        // If it works, unbox it.
        // It if doesn't, take it back.
        user_data
            .downcast()
            .map_err(|user_data| {
                // If we asked for the wrong type, put it back.
                self.user_data = user_data;
            })
            .map(|boxed| *boxed)
            .ok()
    }

    /// Runs the given closure on the stored user data, if any.
    ///
    /// If no user data was supplied, or if the type is different, nothing
    /// will be run.
    ///
    /// Otherwise, the result will be returned.
    pub fn with_user_data<F, T, R>(&mut self, f: F) -> Option<R>
    where
        F: FnOnce(&mut T) -> R,
        T: Any,
    {
        self.user_data().map(f)
    }

    /// Show the debug console.
    ///
    /// Currently, this will show logs if [`logger::init()`](crate::logger::init()) was called.
    pub fn show_debug_console(&mut self) {
        self.add_layer(
            views::Dialog::around(
                views::ScrollView::new(views::NamedView::new(
                    DEBUG_VIEW_NAME,
                    views::DebugView::new(),
                ))
                .scroll_x(true),
            )
            .title("Debug console"),
        );
    }

    /// Show the debug console, or hide it if it's already visible.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use cursive_core::Cursive;
    /// # let mut siv = Cursive::dummy();
    /// siv.add_global_callback('~', Cursive::toggle_debug_console);
    /// ```
    pub fn toggle_debug_console(&mut self) {
        if let Some(pos) =
            self.screen_mut().find_layer_from_name(DEBUG_VIEW_NAME)
        {
            self.screen_mut().remove_layer(pos);
        } else {
            self.show_debug_console();
        }
    }

    /// Returns a sink for asynchronous callbacks.
    ///
    /// Returns the sender part of a channel, that allows to send
    /// callbacks to `self` from other threads.
    ///
    /// Callbacks will be executed in the order
    /// of arrival on the next event cycle.
    ///
    /// # Notes
    ///
    /// Callbacks need to be `Send`, which can be limiting in some cases.
    ///
    /// In some case [`send_wrapper`] may help you work around that.
    ///
    /// [`send_wrapper`]: https://crates.io/crates/send_wrapper
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use cursive_core::*;
    /// let mut siv = Cursive::dummy();
    ///
    /// // quit() will be called during the next event cycle
    /// siv.cb_sink().send(Box::new(|s| s.quit())).unwrap();
    /// ```
    pub fn cb_sink(&self) -> &CbSink {
        &self.cb_sink
    }

    /// Selects the menubar.
    pub fn select_menubar(&mut self) {
        self.menubar.take_focus(direction::Direction::none());
    }

    /// Sets the menubar autohide feature.
    ///
    /// * When enabled (default), the menu is only visible when selected.
    /// * When disabled, the menu is always visible and reserves the top row.
    pub fn set_autohide_menu(&mut self, autohide: bool) {
        self.menubar.autohide = autohide;
    }

    /// Access the menu tree used by the menubar.
    ///
    /// This allows to add menu items to the menubar.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use cursive_core::{Cursive, event};
    /// # use cursive_core::views::{Dialog};
    /// # use cursive_core::traits::*;
    /// # use cursive_core::menu::*;
    /// #
    /// let mut siv = Cursive::dummy();
    ///
    /// siv.menubar()
    ///    .add_subtree("File",
    ///         MenuTree::new()
    ///             .leaf("New", |s| s.add_layer(Dialog::info("New file!")))
    ///             .subtree("Recent", MenuTree::new().with(|tree| {
    ///                 for i in 1..100 {
    ///                     tree.add_leaf(format!("Item {}", i), |_| ())
    ///                 }
    ///             }))
    ///             .delimiter()
    ///             .with(|tree| {
    ///                 for i in 1..10 {
    ///                     tree.add_leaf(format!("Option {}", i), |_| ());
    ///                 }
    ///             })
    ///             .delimiter()
    ///             .leaf("Quit", |s| s.quit()))
    ///    .add_subtree("Help",
    ///         MenuTree::new()
    ///             .subtree("Help",
    ///                      MenuTree::new()
    ///                          .leaf("General", |s| {
    ///                              s.add_layer(Dialog::info("Help message!"))
    ///                          })
    ///                          .leaf("Online", |s| {
    ///                              s.add_layer(Dialog::info("Online help?"))
    ///                          }))
    ///             .leaf("About",
    ///                   |s| s.add_layer(Dialog::info("Cursive v0.0.0"))));
    ///
    /// siv.add_global_callback(event::Key::Esc, |s| s.select_menubar());
    /// ```
    pub fn menubar(&mut self) -> &mut views::Menubar {
        &mut self.menubar
    }

    /// Returns the currently used theme.
    pub fn current_theme(&self) -> &theme::Theme {
        &self.theme
    }

    /// Sets the current theme.
    pub fn set_theme(&mut self, theme: theme::Theme) {
        self.theme = theme;
        self.clear();
    }

    /// Updates the current theme.
    pub fn update_theme(&mut self, f: impl FnOnce(&mut theme::Theme)) {
        // We don't just expose a `current_theme_mut` because we may want to
        // run some logic _after_ setting the theme.
        // Though right now, it's only clearing the screen, so...
        let mut theme = self.theme.clone();
        f(&mut theme);
        self.set_theme(theme);
    }

    /// Clears the screen.
    ///
    /// Users rarely have to call this directly.
    pub fn clear(&mut self) {
        self.backend
            .clear(self.theme.palette[theme::PaletteColor::Background]);
    }

    /// Loads a theme from the given file.
    ///
    /// `filename` must point to a valid toml file.
    ///
    /// Must have the `toml` feature enabled.
    #[cfg(feature = "toml")]
    pub fn load_theme_file<P: AsRef<Path>>(
        &mut self,
        filename: P,
    ) -> Result<(), theme::Error> {
        theme::load_theme_file(filename).map(|theme| self.set_theme(theme))
    }

    /// Loads a theme from the given string content.
    ///
    /// Content must be valid toml.
    ///
    /// Must have the `toml` feature enabled.
    #[cfg(feature = "toml")]
    pub fn load_toml(&mut self, content: &str) -> Result<(), theme::Error> {
        theme::load_toml(content).map(|theme| self.set_theme(theme))
    }

    /// Sets the refresh rate, in frames per second.
    ///
    /// Note that the actual frequency is not guaranteed.
    ///
    /// Between 0 and 30. Call with `fps = 0` to disable (default value).
    pub fn set_fps(&mut self, fps: u32) {
        self.fps = NonZeroU32::new(fps);
    }

    /// Enables or disables automatic refresh of the screen.
    ///
    /// This is a shortcut to call `set_fps` with `30` or `0` depending on
    /// `autorefresh`.
    pub fn set_autorefresh(&mut self, autorefresh: bool) {
        self.set_fps(if autorefresh { 30 } else { 0 });
    }

    /// Returns a reference to the currently active screen.
    pub fn screen(&self) -> &views::StackView {
        self.root.get_inner().screen().unwrap()
    }

    /// Returns a mutable reference to the currently active screen.
    pub fn screen_mut(&mut self) -> &mut views::StackView {
        self.root.get_inner_mut().screen_mut().unwrap()
    }

    /// Returns the id of the currently active screen.
    pub fn active_screen(&self) -> ScreenId {
        self.root.get_inner().active_screen()
    }

    /// Adds a new screen, and returns its ID.
    pub fn add_screen(&mut self) -> ScreenId {
        self.root
            .get_inner_mut()
            .add_screen(views::StackView::new())
    }

    /// Convenient method to create a new screen, and set it as active.
    pub fn add_active_screen(&mut self) -> ScreenId {
        let res = self.add_screen();
        self.set_screen(res);
        res
    }

    /// Sets the active screen. Panics if no such screen exist.
    pub fn set_screen(&mut self, screen_id: ScreenId) {
        self.root.get_inner_mut().set_active_screen(screen_id);
    }

    /// Tries to find the view pointed to by the given selector.
    ///
    /// Runs a closure on the view once it's found, and return the
    /// result.
    ///
    /// If the view is not found, or if it is not of the asked type,
    /// returns None.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use cursive_core::{Cursive, views, view};
    /// # use cursive_core::traits::*;
    /// let mut siv = Cursive::dummy();
    ///
    /// siv.add_layer(views::TextView::new("Text #1").with_name("text"));
    ///
    /// siv.add_global_callback('p', |s| {
    ///     s.call_on(
    ///         &view::Selector::Id("text"),
    ///         |view: &mut views::TextView| {
    ///             view.set_content("Text #2");
    ///         },
    ///     );
    /// });
    /// ```
    pub fn call_on<V, F, R>(
        &mut self,
        sel: &view::Selector<'_>,
        callback: F,
    ) -> Option<R>
    where
        V: View,
        F: FnOnce(&mut V) -> R,
    {
        self.root.call_on(sel, callback)
    }

    /// Tries to find the view identified by the given id.
    ///
    /// Convenient method to use `call_on` with a `view::Selector::Id`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use cursive_core::{Cursive, views};
    /// # use cursive_core::traits::*;
    /// let mut siv = Cursive::dummy();
    ///
    /// siv.add_layer(views::TextView::new("Text #1")
    ///                               .with_name("text"));
    ///
    /// siv.add_global_callback('p', |s| {
    ///     s.call_on_name("text", |view: &mut views::TextView| {
    ///         view.set_content("Text #2");
    ///     });
    /// });
    /// ```
    pub fn call_on_name<V, F, R>(
        &mut self,
        name: &str,
        callback: F,
    ) -> Option<R>
    where
        V: View,
        F: FnOnce(&mut V) -> R,
    {
        self.call_on(&view::Selector::Name(name), callback)
    }

    /// Same as [`call_on_name`](Cursive::call_on_name).
    #[deprecated(note = "`call_on_id` is being renamed to `call_on_name`")]
    pub fn call_on_id<V, F, R>(&mut self, id: &str, callback: F) -> Option<R>
    where
        V: View,
        F: FnOnce(&mut V) -> R,
    {
        self.call_on_name(id, callback)
    }

    /// Convenient method to find a view wrapped in [`NamedView`].
    ///
    /// This looks for a `NamedView<V>` with the given name, and return
    /// a [`ViewRef`] to the wrapped view. The `ViewRef` implements
    /// `DerefMut<Target=T>`, so you can treat it just like a `&mut T`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use cursive_core::Cursive;
    /// # use cursive_core::views::{TextView, ViewRef};
    /// # let mut siv = Cursive::dummy();
    /// use cursive_core::traits::Identifiable;
    ///
    /// siv.add_layer(TextView::new("foo").with_name("id"));
    ///
    /// // Could be called in a callback
    /// let mut view: ViewRef<TextView> = siv.find_name("id").unwrap();
    /// view.set_content("bar");
    /// ```
    ///
    /// Note that you must specify the exact type for the view you're after; for example, using the
    /// wrong item type in a `SelectView` will not find anything:
    ///
    /// ```rust
    /// # use cursive_core::Cursive;
    /// # use cursive_core::views::{SelectView};
    /// # let mut siv = Cursive::dummy();
    /// use cursive_core::traits::Identifiable;
    ///
    /// let select = SelectView::new().item("zero", 0u32).item("one", 1u32);
    /// siv.add_layer(select.with_name("select"));
    ///
    /// // Specifying a wrong type will not return anything.
    /// assert!(siv.find_name::<SelectView<String>>("select").is_none());
    ///
    /// // Omitting the type will use the default type, in this case `String`.
    /// assert!(siv.find_name::<SelectView>("select").is_none());
    ///
    /// // But with the correct type, it works fine.
    /// assert!(siv.find_name::<SelectView<u32>>("select").is_some());
    /// ```
    ///
    /// [`NamedView`]: views::NamedView
    /// [`ViewRef`]: views::ViewRef
    pub fn find_name<V>(&mut self, id: &str) -> Option<views::ViewRef<V>>
    where
        V: View,
    {
        self.call_on_name(id, views::NamedView::<V>::get_mut)
    }

    /// Same as [`find_name`](Cursive::find_name).
    #[deprecated(note = "`find_id` is being renamed to `find_name`")]
    pub fn find_id<V>(&mut self, id: &str) -> Option<views::ViewRef<V>>
    where
        V: View,
    {
        self.find_name(id)
    }

    /// Moves the focus to the view identified by `name`.
    ///
    /// Convenient method to call `focus` with a [`view::Selector::Name`].
    pub fn focus_name(&mut self, name: &str) -> Result<(), ()> {
        self.focus(&view::Selector::Name(name))
    }

    /// Same as [`focus_name`](Cursive::focus_name).
    #[deprecated(note = "`focus_id` is being renamed to `focus_name`")]
    pub fn focus_id(&mut self, id: &str) -> Result<(), ()> {
        self.focus(&view::Selector::Name(id))
    }

    /// Moves the focus to the view identified by `sel`.
    pub fn focus(&mut self, sel: &view::Selector<'_>) -> Result<(), ()> {
        self.root.focus_view(sel)
    }

    /// Adds a global callback.
    ///
    /// Will be triggered on the given key press when no view catches it.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use cursive_core::*;
    /// let mut siv = Cursive::dummy();
    ///
    /// siv.add_global_callback('q', |s| s.quit());
    /// ```
    pub fn add_global_callback<F, E: Into<Event>>(&mut self, event: E, cb: F)
    where
        F: FnMut(&mut Cursive) + 'static,
    {
        self.set_on_post_event(event.into(), cb);
    }

    /// Registers a callback for ignored events.
    ///
    /// This is the same as `add_global_callback`, but can register any `EventTrigger`.
    pub fn set_on_post_event<F, E>(&mut self, trigger: E, cb: F)
    where
        F: FnMut(&mut Cursive) + 'static,
        E: Into<crate::event::EventTrigger>,
    {
        self.root.set_on_event(trigger, crate::immut1!(cb));
    }

    /// Registers a priotity callback.
    ///
    /// If an event matches the given trigger, it will not be sent to the view
    /// tree and will go to the given callback instead.
    ///
    /// Note that regular "post-event" callbacks will also be skipped for
    /// these events.
    pub fn set_on_pre_event<F, E>(&mut self, trigger: E, cb: F)
    where
        F: FnMut(&mut Cursive) + 'static,
        E: Into<crate::event::EventTrigger>,
    {
        self.root.set_on_pre_event(trigger, crate::immut1!(cb));
    }

    /// Registers an inner priority callback.
    ///
    /// See [`OnEventView`] for more information.
    ///
    /// [`OnEventView`]: crate::views::OnEventView::set_on_pre_event_inner()
    pub fn set_on_pre_event_inner<E, F>(&mut self, trigger: E, cb: F)
    where
        E: Into<crate::event::EventTrigger>,
        F: Fn(&Event) -> Option<EventResult> + 'static,
    {
        self.root
            .set_on_pre_event_inner(trigger, move |_, event| cb(event));
    }

    /// Registers an inner callback.
    ///
    /// See [`OnEventView`] for more information.
    ///
    /// [`OnEventView`]: crate::views::OnEventView::set_on_event_inner()
    pub fn set_on_event_inner<E, F>(&mut self, trigger: E, cb: F)
    where
        E: Into<crate::event::EventTrigger>,
        F: Fn(&Event) -> Option<EventResult> + 'static,
    {
        self.root
            .set_on_event_inner(trigger, move |_, event| cb(event));
    }

    /// Sets the only global callback for the given event.
    ///
    /// Any other callback for this event will be removed.
    ///
    /// See also [`Cursive::add_global_callback`].
    pub fn set_global_callback<F, E: Into<Event>>(&mut self, event: E, cb: F)
    where
        F: FnMut(&mut Cursive) + 'static,
    {
        let event = event.into();
        self.clear_global_callbacks(event.clone());
        self.add_global_callback(event, cb);
    }

    /// Fetches the type name of a view in the tree.
    pub fn debug_name(&mut self, name: &str) -> Option<&'static str> {
        let mut result = None;

        self.root.call_on_any(
            &view::Selector::Name(name),
            &mut |v: &mut dyn crate::View| result = Some(v.type_name()),
        );
        result
    }

    /// Removes any callback tied to the given event.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cursive_core::Cursive;
    /// let mut siv = Cursive::dummy();
    ///
    /// siv.add_global_callback('q', |s| s.quit());
    /// siv.clear_global_callbacks('q');
    /// ```
    pub fn clear_global_callbacks<E>(&mut self, event: E)
    where
        E: Into<Event>,
    {
        let event = event.into();
        self.root.clear_event(event);
    }

    /// This resets the default callbacks.
    ///
    /// Currently this mostly includes exiting on Ctrl-C.
    pub fn reset_default_callbacks(&mut self) {
        self.set_on_pre_event(Event::CtrlChar('c'), |s| s.quit());
        self.set_on_pre_event(Event::Exit, |s| s.quit());

        self.set_on_pre_event(Event::WindowResize, |s| s.clear());
    }

    /// Add a layer to the current screen.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cursive_core::{Cursive, views};
    /// let mut siv = Cursive::dummy();
    ///
    /// siv.add_layer(views::TextView::new("Hello world!"));
    /// ```
    pub fn add_layer<T>(&mut self, view: T)
    where
        T: IntoBoxedView,
    {
        self.screen_mut().add_layer(view);
    }

    /// Adds a new full-screen layer to the current screen.
    ///
    /// Fullscreen layers have no shadow.
    pub fn add_fullscreen_layer<T>(&mut self, view: T)
    where
        T: IntoBoxedView,
    {
        self.screen_mut().add_fullscreen_layer(view);
    }

    /// Convenient method to remove a layer from the current screen.
    pub fn pop_layer(&mut self) -> Option<Box<dyn View>> {
        self.screen_mut().pop_layer()
    }

    /// Convenient stub forwarding layer repositioning.
    pub fn reposition_layer(
        &mut self,
        layer: LayerPosition,
        position: Position,
    ) {
        self.screen_mut().reposition_layer(layer, position);
    }

    /// Processes an event.
    ///
    /// * If the menubar is active, it will be handled the event.
    /// * The view tree will be handled the event.
    /// * If ignored, global_callbacks will be checked for this event.
    pub fn on_event(&mut self, event: Event) {
        if let Event::Mouse {
            event, position, ..
        } = event
        {
            if event.grabs_focus()
                && !self.menubar.autohide
                && !self.menubar.has_submenu()
                && position.y == 0
            {
                self.select_menubar();
            }
        }

        if self.menubar.receive_events() {
            self.menubar.on_event(event).process(self);
        } else {
            let offset = if self.menubar.autohide { 0 } else { 1 };

            let result =
                View::on_event(&mut self.root, event.relativized((0, offset)));

            if let EventResult::Consumed(Some(cb)) = result {
                cb(self);
            }
        }
    }

    /// Returns the size of the screen, in characters.
    pub fn screen_size(&self) -> Vec2 {
        self.backend.screen_size()
    }

    fn layout(&mut self) {
        let size = self.screen_size();
        let offset = if self.menubar.autohide { 0 } else { 1 };
        let size = size.saturating_sub((0, offset));
        self.root.layout(size);
    }

    fn draw(&mut self) {
        // TODO: do not allocate in the default, fast path?
        let sizes = self.screen().layer_sizes();
        if self.last_sizes != sizes {
            // TODO: Maybe we only need to clear if the _max_ size differs?
            // Or if the positions change?
            self.clear();
            self.last_sizes = sizes;
        }

        let printer =
            Printer::new(self.screen_size(), &self.theme, &*self.backend);

        let selected = self.menubar.receive_events();

        // Print the stackview background before the menubar
        let offset = if self.menubar.autohide { 0 } else { 1 };

        let sv_printer = printer.offset((0, offset)).focused(!selected);
        self.root.draw(&sv_printer);

        self.root.get_inner().draw_bg(&sv_printer);

        // Draw the currently active screen
        // If the menubar is active, nothing else can be.
        // Draw the menubar?
        if self.menubar.visible() {
            let printer = printer.focused(self.menubar.receive_events());
            self.menubar.draw(&printer);
        }

        // finally draw stackview layers
        // using variables from above
        self.root.get_inner().draw_fg(&sv_printer);
    }

    /// Returns `true` until [`quit(&mut self)`] is called.
    ///
    /// [`quit(&mut self)`]: #method.quit
    pub fn is_running(&self) -> bool {
        self.running
    }

    /// Runs the event loop.
    ///
    /// It will wait for user input (key presses)
    /// and trigger callbacks accordingly.
    ///
    /// Internally, it calls [`step(&mut self)`] until [`quit(&mut self)`] is
    /// called.
    ///
    /// After this function returns, you can call it again and it will start a
    /// new loop.
    ///
    /// [`step(&mut self)`]: #method.step
    /// [`quit(&mut self)`]: #method.quit
    pub fn run(&mut self) {
        self.running = true;

        self.refresh();

        // And the big event loop begins!
        while self.running {
            self.step();
        }
    }

    /// Performs a single step from the event loop.
    ///
    /// Useful if you need tighter control on the event loop.
    /// Otherwise, [`run(&mut self)`] might be more convenient.
    ///
    /// Returns `true` if an input event or callback was received
    /// during this step, and `false` otherwise.
    ///
    /// [`run(&mut self)`]: #method.run
    pub fn step(&mut self) -> bool {
        let received_something = self.process_events();
        self.post_events(received_something);
        received_something
    }

    /// Performs the first half of `Self::step()`.
    ///
    /// This is an advanced method for fine-tuned manual stepping;
    /// you probably want [`run`][1] or [`step`][2].
    ///
    /// This processes any pending event or callback. After calling this,
    /// you will want to call [`post_events`][3] with the result from this
    /// function.
    ///
    /// Returns `true` if an event or callback was received,
    /// and `false` otherwise.
    ///
    /// [1]: Cursive::run()
    /// [2]: Cursive::step()
    /// [3]: Cursive::post_events()
    pub fn process_events(&mut self) -> bool {
        // Things are boring if nothing significant happened.
        let mut boring = true;

        // First, handle all available input
        while let Some(event) = self.backend.poll_event() {
            boring = false;
            self.on_event(event);

            if !self.running {
                return true;
            }
        }

        // Then, handle any available callback
        while let Ok(cb) = self.cb_source.try_recv() {
            boring = false;
            cb(self);

            if !self.running {
                return true;
            }
        }

        !boring
    }

    /// Performs the second half of `Self::step()`.
    ///
    /// This is an advanced method for fine-tuned manual stepping;
    /// you probably want [`run`][1] or [`step`][2].
    ///
    /// You should call this after [`process_events`][3].
    ///
    /// [1]: Cursive::run()
    /// [2]: Cursive::step()
    /// [3]: Cursive::process_events()
    pub fn post_events(&mut self, received_something: bool) {
        let boring = !received_something;
        // How many times should we try if it's still boring?
        // Total duration will be INPUT_POLL_DELAY_MS * repeats
        // So effectively fps = 1000 / INPUT_POLL_DELAY_MS / repeats
        if !boring
            || self
                .fps
                .map(|fps| 1000 / INPUT_POLL_DELAY_MS as u32 / fps.get())
                .map(|repeats| self.boring_frame_count >= repeats)
                .unwrap_or(false)
        {
            // We deserve to draw something!

            if boring {
                // We're only here because of a timeout.
                self.on_event(Event::Refresh);
            }

            self.refresh();
        }

        if boring {
            std::thread::sleep(Duration::from_millis(INPUT_POLL_DELAY_MS));
            self.boring_frame_count += 1;
        }
    }

    /// Refresh the screen with the current view tree state.
    pub fn refresh(&mut self) {
        self.boring_frame_count = 0;

        // Do we need to redraw everytime?
        // Probably, actually.
        // TODO: Do we need to re-layout everytime?
        self.layout();

        // TODO: Do we need to redraw every view every time?
        // (Is this getting repetitive? :p)
        self.draw();
        self.backend.refresh();
    }

    /// Stops the event loop.
    pub fn quit(&mut self) {
        self.running = false;
    }

    /// Does not do anything.
    pub fn noop(&mut self) {
        // foo
    }

    /// Return the name of the backend used.
    ///
    /// Mostly used for debugging.
    pub fn backend_name(&self) -> &str {
        self.backend.name()
    }
}

impl Drop for Cursive {
    fn drop(&mut self) {
        self.backend.finish();
    }
}
