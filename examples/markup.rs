use cursive::theme::{BaseColor, Color, Effect, Style};
use cursive::utils::markup::StyledString;
use cursive::views;
use cursive::Cursive;

fn main() {
    let mut siv = Cursive::default();

    let mut styled = StyledString::plain("Isn't ");
    styled.append(StyledString::styled("that ", Color::Dark(BaseColor::Red)));
    styled.append(StyledString::styled(
        "cool?",
        Style::from(Color::Light(BaseColor::Blue)).combine(Effect::Bold),
    ));

    // views::Text can natively accept StyledString.
    siv.add_layer(
        views::Dialog::around(views::Text::new(styled))
            .button("Hell yeah!", |s| s.quit()),
    );

    siv.run();
}
