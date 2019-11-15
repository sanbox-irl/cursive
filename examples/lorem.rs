use cursive::align::HAlign;
use cursive::traits::Scrollable as _;
use cursive::{views, Cursive};

fn main() {
    // Read some long text from a file.
    let content = include_str!("../assets/lorem.txt");

    let mut siv = Cursive::default();

    // We can quit by pressing q
    siv.add_global_callback('q', |s| s.quit());

    // The text is too long to fit on a line, so the view will wrap lines,
    // and will adapt to the terminal size.
    siv.add_fullscreen_layer(
        views::Dialog::around(views::Panel::new(
            views::Text::new(content).scrollable(),
        ))
        .title("Unicode and wide-character support")
        // This is the alignment for the button
        .h_align(HAlign::Center)
        .button("Quit", |s| s.quit()),
    );
    // Show a popup on top of the view.
    siv.add_layer(views::Dialog::info(
        "Try resizing the terminal!\n(Press 'q' to \
         quit when you're done.)",
    ));

    siv.run();
}
