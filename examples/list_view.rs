use cursive::traits::*;
use cursive::views;
use cursive::Cursive;

// This example uses a views::List.
//
// views::List can be used to build forms, with a list of inputs.

fn main() {
    let mut siv = Cursive::default();

    siv.add_layer(
        views::Dialog::new()
            .title("Please fill out this form")
            .button("Ok", |s| s.quit())
            .content(
                views::List::new()
                    // Each child is a single-line view with a label
                    .child("Name", views::Edit::new().fixed_width(10))
                    .child(
                        "Receive spam?",
                        views::Checkbox::new().on_change(|s, checked| {
                            // Enable/Disable the next field depending on this checkbox
                            for name in &["email1", "email2"] {
                                s.call_on_id(
                                    name,
                                    |view: &mut views::Edit| {
                                        view.set_enabled(checked)
                                    },
                                );
                                if checked {
                                    s.focus_id("email1").unwrap();
                                }
                            }
                        }),
                    )
                    .child(
                        "Email",
                        // Each child must have a height of 1 line,
                        // but we can still combine multiple views!
                        views::LinearLayout::horizontal()
                            .child(
                                views::Edit::new()
                                    .disabled()
                                    .with_id("email1")
                                    .fixed_width(15),
                            )
                            .child(views::Text::new("@"))
                            .child(
                                views::Edit::new()
                                    .disabled()
                                    .with_id("email2")
                                    .fixed_width(10),
                            ),
                    )
                    // Delimiter currently are just a blank line
                    .delimiter()
                    .child(
                        "Age",
                        // Popup-mode views::Select are small enough to fit here
                        views::Select::new()
                            .popup()
                            .item_str("0-18")
                            .item_str("19-30")
                            .item_str("31-40")
                            .item_str("41+"),
                    )
                    .with(|list| {
                        // We can also add children procedurally
                        for i in 0..50 {
                            list.add_child(
                                &format!("Item {}", i),
                                views::Edit::new(),
                            );
                        }
                    })
                    .scrollable(),
            ),
    );

    siv.run();
}
