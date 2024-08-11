use {scaffolding::prelude::*, scaffolding_tui::prelude::*};

fn main() {
    let mut world = World::new();
    world.add_plugin(ScaffoldingTuiPlugin::default());

    TuiRunloop::default().start(world, app_main);
}

fn app_main(terminal: &Singleton<Terminal>, msg_sender: &mut TerminalMsgSender) {
    terminal.draw(Border {
        x: 0,
        y: 0,
        width: 50,
        height: 30,
        fg: None,
        bg: None,
        style: BorderStyle::ROUND,
    });
    terminal.draw(Text {
        x: 1,
        y: 1,
        fg: None,
        bg: None,
        text: format!("Terminal size: {:?}", terminal.size),
    });
    terminal.draw(Text {
        x: 1,
        y: 2,
        fg: None,
        bg: None,
        text: format!("Mouse pos: {:?}", terminal.mouse_pos),
    });
    terminal.draw(Rect {
        x: terminal.mouse_pos.0,
        y: terminal.mouse_pos.1,
        width: 1,
        height: 1,
        colour: Some(Colour::BLUE),
    });

    if terminal.pressed_keys.contains(&Key::Escape) {
        msg_sender.send_exit_tui_runloop();
    }

    msg_sender.send_update();
}
