use {scaffolding::prelude::*, scaffolding_tui::prelude::*};

struct AppState {
    key_log: String,
}

enum AppMsg {
    KeyPress(String),
}

fn main() {
    let mut world = World::new();
    world
        .add_plugin(TuiPlugin::default())
        .add_singleton(AppState {
            key_log: String::from("Key log: "),
        })
        .add_msg_handler(app_msg_handler);

    TuiRunloop::default().start(world, app_main);
}

fn app_main(
    terminal: &Singleton<Terminal>,
    msg_sender: &MsgSender,
    app: &App,
    app_state: &Singleton<AppState>,
    uniqs: &Uniqs,
) {
    let buffer = uniqs.get(uniq_key!());
    app.draw(TextInput::new(buffer, uniq_key!()).x(50).width(30));

    terminal.draw(Border {
        x: 0,
        y: 0,
        width: 50,
        height: 30,
        style: BorderStyle::ROUND,
    });
    terminal.draw(RawString {
        x: 1,
        y: 1,
        text: format!("Terminal size: {:?}", terminal.size),
    });
    terminal.draw(RawString {
        x: 1,
        y: 2,
        text: format!("Mouse pos: {:?}", terminal.mouse_pos),
    });
    terminal.draw(RawString {
        x: 1,
        y: 3,
        text: format!(
            "Pressed mouse buttons: {:?}",
            terminal.pressed_mouse_buttons
        ),
    });
    terminal.draw(RawString {
        x: 1,
        y: 4,
        text: &app_state.key_log,
    });

    terminal.set_fg(Some(Colour::BLUE));
    terminal.draw(Rect {
        x: terminal.mouse_pos.0,
        y: terminal.mouse_pos.1,
        width: 1,
        height: 1,
    });

    if terminal.pressed_keys.contains(&Key::Escape) {
        app.exit();
    }
    for key in terminal.pressed_keys.iter() {
        msg_sender.send(AppMsg::KeyPress(key.to_string()));
    }
}

fn app_msg_handler(world: &mut World, msg: Msg<AppMsg>) {
    match msg.read() {
        AppMsg::KeyPress(key) => {
            let state: &mut AppState = world.get_singleton_mut();
            state.key_log += &key;
        }
    }
}
