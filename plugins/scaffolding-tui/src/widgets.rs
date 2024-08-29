use {
    crate::{
        input::{Key, ScrollDirection},
        prelude::Terminal,
        shapes::{Border, BorderStyle, Text},
        Colour,
    },
    scaffolding::{
        datatypes::uniq::UniqKey,
        world::{Executable, IntoStatefulExecutable, Singleton, State, StatesStorage},
    },
    std::{
        marker::PhantomData,
        ops::{Deref, DerefMut},
    },
    unicode_segmentation::UnicodeSegmentation,
};

pub struct WidgetState<S> {
    pub frame: Frame,
    pub fg: Option<Colour>,
    pub bg: Option<Colour>,
    pub state: S,
}
impl<S> WidgetState<S> {
    pub fn new(state: S) -> Self {
        Self {
            frame: Default::default(),
            fg: Some(Colour::WHITE),
            bg: None,
            state,
        }
    }
}
impl<S> Deref for WidgetState<S> {
    type Target = S;

    fn deref(&self) -> &Self::Target {
        &self.state
    }
}
impl<S> DerefMut for WidgetState<S> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.state
    }
}

#[derive(Default)]
pub struct Frame {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}
impl Frame {
    pub fn contains(&self, pos: (u16, u16)) -> bool {
        let (x, y) = pos;
        x > self.x && x < (self.x + self.width) && y > self.y && y < (self.y + self.height)
    }
}

struct Widget<'a, IE: IntoStatefulExecutable<'a, Args, State = WidgetState<State>>, Args, State> {
    executable: IE,
    state: WidgetState<State>,
    _args: PhantomData<&'a Args>,
}
impl<'a, IE: IntoStatefulExecutable<'a, Args, State = WidgetState<State>>, State, Args>
    Widget<'a, IE, Args, State>
{
    pub fn new(func: IE, state: State) -> Self {
        Self {
            executable: func,
            state: WidgetState::new(state),
            _args: PhantomData,
        }
    }
}

pub trait SomeWidget<'a> {
    type Output: 'a;

    fn x(self, x: u16) -> Self;
    fn y(self, y: u16) -> Self;
    fn width(self, width: u16) -> Self;
    fn height(self, height: u16) -> Self;

    fn fg(self, fg: Option<Colour>) -> Self;
    fn bg(self, bg: Option<Colour>) -> Self;

    fn build(self) -> impl Executable<'a, Output = Self::Output>;
}
impl<'a, IE: IntoStatefulExecutable<'a, Args, State = WidgetState<State>>, Args, State>
    SomeWidget<'a> for Widget<'a, IE, Args, State>
{
    type Output = IE::Output;

    fn x(mut self, x: u16) -> Self {
        self.state.frame.x = x;
        self
    }
    fn y(mut self, y: u16) -> Self {
        self.state.frame.y = y;
        self
    }
    fn width(mut self, width: u16) -> Self {
        self.state.frame.width = width;
        self
    }
    fn height(mut self, height: u16) -> Self {
        self.state.frame.height = height;
        self
    }

    fn bg(mut self, bg: Option<Colour>) -> Self {
        self.state.bg = bg;
        self
    }
    fn fg(mut self, fg: Option<Colour>) -> Self {
        self.state.fg = fg;
        self
    }

    fn build(self) -> impl Executable<'a, Output = Self::Output> {
        self.executable.into_executable_with_state(self.state)
    }
}

#[derive(Default)]
pub struct ButtonOut {
    pub clicked: bool,
}
pub fn button() -> impl SomeWidget<'static> {
    Widget::new(_button, ())
}
fn _button(state: State<WidgetState<()>>, terminal: &Singleton<Terminal>) -> ButtonOut {
    let mut out = ButtonOut::default();

    if terminal.pressed_mouse_buttons.contains(&0) && state.frame.contains(terminal.mouse_pos) {
        out.clicked = true;
    }

    out
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
pub fn text_input(buffer: &mut String, cache_key: UniqKey) -> impl SomeWidget {
    Widget::new(_text_input, TextInputState { buffer, cache_key })
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
        cached_state.focused = state.frame.contains(terminal.mouse_pos);
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
            (state.frame.x + cached_state.cursor_pos as u16).saturating_add(1),
            state.frame.y + 1,
        )));
    }

    terminal.draw(Border {
        x: state.frame.x,
        y: state.frame.y,
        width: state.frame.width,
        height: 3,
        fg: state.fg,
        bg: state.bg,
        style: BorderStyle::ROUND,
    });
    terminal.draw(Text {
        x: state.frame.x + 1,
        y: state.frame.y + 1,
        max_width: Some(state.frame.width - 2),
        max_height: Some(1),
        fg: state.fg,
        bg: state.bg,
        text: &*state.buffer,
    })
}
