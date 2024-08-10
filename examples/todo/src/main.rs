use {
    scaffolding::prelude::*,
    scaffolding_tui::prelude::*,
    std::{thread, time::Duration},
};

fn main() {
    let mut world = World::new();
    world.add_plugin(ScaffoldingTuiPlugin::default());

    loop {
        world.execute(|term: &Singleton<Terminal>, cb: &mut PostRunCallback<_>| {
            term.reset();

            term.draw(Border {
                x: 0,
                y: 0,
                width: 50,
                height: 30,
                fg: None,
                bg: None,
                style: BorderStyle::ROUND,
            });
            term.draw(Text {
                x: 1,
                y: 1,
                fg: None,
                bg: None,
                text: format!("Terminal size: {:?}", term.size),
            });
            term.draw(Text {
                x: 1,
                y: 2,
                fg: None,
                bg: None,
                text: format!("Mouse pos: {:?}", term.mouse_pos),
            });
            term.draw(Rect {
                x: term.mouse_pos.0,
                y: term.mouse_pos.1,
                width: 1,
                height: 1,
                colour: Some(Colour::BLUE),
            });

            term.flush();

            cb.set_callback(|world| world.send_msg(MsgUpdateTerminal));
        });

        thread::sleep(Duration::from_secs_f64(1.0 / 60.0));
    }
}
