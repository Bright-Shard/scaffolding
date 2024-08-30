//! Standard widgets shipped by Scaffolding TUI

use {
    super::api::*,
    crate::prelude::*,
    scaffolding::{datatypes::uniq::UniqKey, plugin_prelude::*},
    unicode_segmentation::UnicodeSegmentation,
};

/// A clickable button.
pub fn button(
    label: &str,
    cache_key: UniqKey,
) -> impl SomeWidget<Output = ButtonOut> + PositionedWidget {
    Widget::new(
        _button,
        ButtonState {
            label,
            cache_key: Some(cache_key),
        },
    )
}
#[derive(Default)]
struct ButtonState<'a> {
    label: &'a str,
    cache_key: Option<UniqKey>,
}
#[derive(Default, Clone, Copy, Debug)]
pub struct ButtonOut {
    /// If the user has pressed down on the button. This triggers on mouse-down
    /// and resets to false after the first frame.
    pub pressed: bool,
    /// If the user is currently pressing on the button. This triggers on
    /// mouse-down and stays true until the user releases the mouse button. This
    /// will become false if the user moves their cursor outside of the button.
    pub held: bool,
    /// Triggers on mouse-up over the button (ie, when the user stops clicking
    /// on the button). This will not trigger if the user moves their cursor
    /// outside of the button.
    pub released: bool,
}
fn _button(
    mut state: State<WidgetState<ButtonState>>,
    terminal: &Singleton<Terminal>,
    states_storage: &StatesStorage,
) -> ButtonOut {
    let cached_state: &mut ButtonOut = states_storage.get(state.cache_key.take().unwrap());

    if cached_state.released {
        cached_state.released = false;
    }
    if state.contains(terminal.mouse_pos) {
        if terminal.pressed_mouse_buttons.contains(&0) {
            if cached_state.pressed {
                cached_state.pressed = false;
                cached_state.held = true;
            } else if !cached_state.held {
                cached_state.pressed = true;
            }
        } else if cached_state.pressed || cached_state.held {
            cached_state.pressed = false;
            cached_state.held = false;
            cached_state.released = true;
        }
    }

    terminal.draw(Border {
        x: state.x,
        y: state.y,
        width: state.width,
        height: state.height,
        fg: state.fg,
        bg: state.bg,
        style: BorderStyle::ROUND,
    });

    let remaining_x_space = state
        .width
        .saturating_sub(2)
        .saturating_sub(state.label.len() as u16);
    let remaining_y_space = state.height.saturating_sub(3);

    terminal.draw(Text {
        x: state.x + 1 + (remaining_x_space / 2),
        y: state.y + 1 + (remaining_y_space / 2),
        max_width: Some(state.width.saturating_sub(2)),
        max_height: Some(1),
        fg: state.fg,
        bg: state.bg,
        text: state.label,
    });

    *cached_state
}

/// Creates a single-line text input field. The `buffer` string will store the
/// user's input as they type.
pub fn text_input(buffer: &mut String, cache_key: UniqKey) -> impl SomeWidget + PositionedWidget {
    Widget::new(_text_input, TextInputState { buffer, cache_key })
}
struct TextInputState<'a> {
    buffer: &'a mut String,
    cache_key: UniqKey,
}
#[derive(Default)]
struct TextInputCachedState {
    /// The cursor position
    /// This is its actual on-screen position, not a byte index
    cursor_pos: usize,
    /// If the text input is currently focused
    focused: bool,
}
fn _text_input(
    mut state: State<WidgetState<TextInputState<'_>>>,
    terminal: &Singleton<Terminal>,
    states_store: &StatesStorage,
) {
    // Safety: We never use `cache_key` again
    // Could probably replace it with an option and use .take at some point...
    let cached_state: &mut TextInputCachedState =
        states_store.get(unsafe { state.cache_key.clone() });

    if terminal.pressed_mouse_buttons.contains(&0) {
        cached_state.focused = state.contains(terminal.mouse_pos);
    }

    if cached_state.focused {
        for val in terminal.pressed_keys.iter() {
            match val {
                Key::Text(char) => {
                    let byte_idx = state
                        .buffer
                        .grapheme_indices(true)
                        .nth(cached_state.cursor_pos)
                        .map(|(idx, _)| idx)
                        .unwrap_or(state.buffer.len());
                    state.buffer.insert(byte_idx, *char);
                    cached_state.cursor_pos += 1;
                }
                Key::ArrowLeft => {
                    cached_state.cursor_pos = cached_state.cursor_pos.saturating_sub(1);
                }
                Key::ArrowRight => {
                    cached_state.cursor_pos = cached_state
                        .cursor_pos
                        .saturating_add(1)
                        .clamp(0, state.buffer.len());
                }
                Key::Delete => {
                    let byte_idx = state
                        .buffer
                        .grapheme_indices(true)
                        .nth(cached_state.cursor_pos)
                        .map(|(idx, _)| idx)
                        .unwrap_or(state.buffer.len());

                    if byte_idx == state.buffer.len() {
                        continue;
                    }

                    state.buffer.remove(byte_idx);
                }
                Key::Backspace => {
                    if cached_state.cursor_pos == 0 {
                        continue;
                    }

                    let byte_idx = state
                        .buffer
                        .grapheme_indices(true)
                        .nth(cached_state.cursor_pos - 1)
                        .map(|(idx, _)| idx)
                        .unwrap_or(state.buffer.len());
                    state.buffer.remove(byte_idx);
                    cached_state.cursor_pos -= 1;
                }
                _ => {}
            }
        }

        terminal.target_cursor_location.set(Some((
            (state.x + cached_state.cursor_pos as u16).saturating_add(1),
            state.y + 1,
        )));
    }

    terminal.draw(Border {
        x: state.x,
        y: state.y,
        width: state.width,
        height: 3,
        fg: state.fg,
        bg: state.bg,
        style: BorderStyle::ROUND,
    });
    terminal.draw(Text {
        x: state.x + 1,
        y: state.y + 1,
        max_width: Some(state.width - 2),
        max_height: Some(1),
        fg: state.fg,
        bg: state.bg,
        text: &*state.buffer,
    })
}
