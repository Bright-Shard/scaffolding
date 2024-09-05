use {scaffolding::prelude::*, scaffolding_tui::prelude::*};

fn main() {
    let mut world = World::new();
    world.add_plugin(TuiPlugin::default());

    TuiRunloop::new(60).start(world, app);
}

fn app(app: &App, terminal: &Singleton<Terminal>, uniqs: &Uniqs) {
    app.draw(TextInput::new(uniqs.get(uniq_key!()), uniq_key!()));

    let btn = app.draw(
        Button::new("button :D", uniq_key!())
            .x(11)
            .width(15)
            .height(3),
    );
    app.draw(
        Button::new("Tol button :D", uniq_key!())
            .x(27)
            .width(20)
            .height(5),
    );
    app.draw(Checkbox::new("Checkbox", uniq_key!()).x(48).width(10).y(1));

    terminal.draw(RawString {
        x: 0,
        y: 5,
        text: format!("Button is: {:?}", btn),
    });

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
