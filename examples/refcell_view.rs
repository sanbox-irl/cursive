use cursive::traits::{Identifiable, Resizable};
use cursive::views::{Dialog, Edit, LinearLayout, Text};
use cursive::Cursive;

// This example shows a way to access multiple views at the same time.

fn main() {
    let mut siv = Cursive::default();

    // Create a dialog with 2 edit fields, and a text view.
    // The text view indicates when the 2 fields content match.
    siv.add_layer(
        Dialog::around(
            LinearLayout::vertical()
                .child(Edit::new().on_edit(on_edit).with_id("1"))
                .child(Edit::new().on_edit(on_edit).with_id("2"))
                .child(Text::new("match").with_id("match"))
                .fixed_width(10),
        )
        .button("Quit", Cursive::quit),
    );

    siv.run();
}

// Compare the content of the two edit views,
// and update the Text accordingly.
//
// We'll ignore the `content` and `cursor` arguments,
// and directly retrieve the content from the `Cursive` root.
fn on_edit(siv: &mut Cursive, _content: &str, _cursor: usize) {
    // Get handles for each view.
    let edit_1 = siv.find_id::<Edit>("1").unwrap();
    let edit_2 = siv.find_id::<Edit>("2").unwrap();

    // Directly compare references to edit_1 and edit_2.
    let matches = edit_1.get_content() == edit_2.get_content();

    siv.call_on_id("match", |v: &mut Text| {
        v.set_content(if matches { "match" } else { "no match" })
    });
}
