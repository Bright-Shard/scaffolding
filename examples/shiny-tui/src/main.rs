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
    let frames: &mut u128 = uniqs.get(uniq_key!());

    let text_input_buffer = uniqs.get(uniq_key!());
    app.draw(TextInput::new(text_input_buffer, uniq_key!()).placeholder("Text box"));

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
    app.draw(
        Text::new(format!("Frames: {frames}").as_str())
            .x(15)
            .y(6)
            .height(1)
            .width(60)
            .horizontal_overflow(HorizontalOverflowStyle::Clip),
    );
    app.draw(
        Text::new(&format!("Mouse position: {:?}", terminal.mouse_pos))
            .x(0)
            .y(7)
            .height(1)
            .width(60)
            .horizontal_overflow(HorizontalOverflowStyle::Clip),
    );
    app.draw(
        Text::new(&format!("Window size: {:?}", terminal.size))
            .x(0)
            .y(8)
            .height(1)
            .width(60)
            .horizontal_overflow(HorizontalOverflowStyle::Clip),
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
            y: 10,
            width: 7,
            height: 7,
            style: border,
        });
    });

    if terminal.pressed_keys.contains(&Key::Escape) {
        app.exit();
    }

    *frames += 1;
}
