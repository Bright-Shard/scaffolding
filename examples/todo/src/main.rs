use {scaffolding::prelude::*, scaffolding_tui::prelude::*};

struct TodoItem {
    label: String,
    complete: bool,
}

fn main() {
    let mut world = World::new();
    world.add_plugin(TuiPlugin::default());

    TuiRunloop::default().start(world, app_main);
}

fn app_main(terminal: &Singleton<Terminal>, msg_sender: &MsgSender, app: &App, uniqs: &Uniqs) {
    let todo_list: &mut Vec<TodoItem> = uniqs.get(uniq_key!());
    let mut y = 0;

    for (idx, item) in todo_list.iter_mut().enumerate() {
        let checkbox = app.draw(Checkbox::new("", uniq_key!(idx)).width(2).y(y));

        let input = TextInput::new(&mut item.label, uniq_key!(idx))
            .width(terminal.size.0.saturating_sub(2))
            .x(2)
            .y(y)
            .border(None);

        if checkbox.checked {
            app.draw(input.text_style(TextStyle::Strikethrough));
        } else {
            app.draw(input);
        }

        y += 1;
    }

    let add_btn = app.draw(Button::new("+").y(y.saturating_add(1)).width(5).height(3));
    if add_btn.state == ButtonState::Pressed {
        todo_list.push(TodoItem {
            label: String::new(),
            complete: false,
        });
    }

    if terminal.pressed_keys.contains(&Key::Escape) {
        app.exit();
    }
}
