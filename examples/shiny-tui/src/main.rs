use {scaffolding::prelude::*, scaffolding_tui::prelude::*};

fn main() {
    let mut world = World::new();
    world.add_plugin(TuiPlugin::default());

    TuiRunloop::new(60).start(world, app);
}

fn app(app: &App, terminal: &Singleton<Terminal>, states_storage: &StatesStorage) {
    let buffer = states_storage.get(uniq_key!());

    app.draw(text_input(buffer, uniq_key!()).width(10));
    let btn = app.draw(button("Button :D", uniq_key!()).x(11).width(15).height(3));
    app.draw(
        button("Tol Button :D", uniq_key!())
            .x(30)
            .width(20)
            .height(5),
    );

    terminal.draw(Text {
        x: 0,
        y: 5,
        max_width: None,
        max_height: None,
        fg: Some(Colour::WHITE),
        bg: None,
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
            fg: Some(Colour::WHITE),
            bg: None,
            style: border,
        });
    });

    if terminal.pressed_keys.contains(&Key::Escape) {
        app.exit();
    }
}
