use {
    scaffolding::prelude::*,
    scaffolding_tui::{prelude::*, widgets::HorizontalOverflowStyle},
};

fn main() {
    let mut world = World::new();
    world.add_plugin(TuiPlugin::default());

    TuiRunloop::new(60).start(world, app);
}

fn app(app: &App, terminal: &Singleton<Terminal>, uniqs: &Uniqs) {
    app.draw(TextInput::new(uniqs.get(uniq_key!()), uniq_key!()).placeholder("Text box"));

    let btn = app.draw(Button::new("button :D").x(11).width(15).height(3));
    app.draw(Button::new("Tol button :D").x(27).width(20).height(5));
    app.draw(Checkbox::new("Checkbox", uniq_key!()).x(48).width(10).y(1));

    terminal.draw(RawString {
        x: 0,
        y: 5,
        text: format!("Button is: {:?}", btn),
    });
    app.draw(
        Text::new("FANCY TEXT :DDDD")
            .x(0)
            .y(6)
            .height(1)
            .horizontal_overflow(HorizontalOverflowStyle::Clip)
            .text_style(TextStyle::Blinking | TextStyle::Underline),
    );

    [
        BorderStyle::ASCII,
        BorderStyle::DOUBLE,
        BorderStyle::HEAVY,
        BorderStyle::NORMAL,
        BorderStyle::ROUND,
    ]
    .into_iter()
    .enumerate()
    .for_each(|(idx, border)| {
        terminal.draw(Border {
            x: idx as u16 * 7,
            y: 7,
            width: 7,
            height: 7,
            style: border,
        });
    });

    if terminal.pressed_keys.contains(&Key::Escape) {
        app.exit();
    }
}
