use {
    crate::{shapes::BorderStyle, Colour},
    scaffolding::world::{Executable, IntoStatefulExecutable},
    std::{
        marker::PhantomData,
        ops::{Deref, DerefMut},
    },
};

pub enum TextHorizontalAlign {
    Center,
    Left,
    Right,
}
pub enum TextVerticalAlign {
    Center,
    Top,
    Bottom,
}

/// State common to all widgets. This stores basic data like colour, position,
/// and border style.
pub struct WidgetState<S> {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
    pub fg: Option<Colour>,
    pub bg: Option<Colour>,
    pub border_style: Option<BorderStyle>,
    pub text_vertical_align: TextVerticalAlign,
    pub text_horizontal_align: TextHorizontalAlign,
    pub state: S,
}
impl<S> WidgetState<S> {
    pub fn new(state: S) -> Self {
        Self {
            x: 0,
            y: 0,
            width: 10,
            height: 10,
            fg: Some(Colour::WHITE),
            bg: None,
            border_style: None,
            text_vertical_align: TextVerticalAlign::Center,
            text_horizontal_align: TextHorizontalAlign::Center,
            state,
        }
    }

    pub fn contains(&self, point: (u16, u16)) -> bool {
        let (x, y) = point;
        x >= self.x && x < (self.x + self.width) && y >= self.y && y < (self.y + self.height)
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

pub struct Widget<'a, IE: IntoStatefulExecutable<'a, Args, State = WidgetState<State>>, Args, State>
{
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

    fn build(self) -> impl Executable<'a, Output = Self::Output>;
}
pub trait PositionedWidget<'a>: SomeWidget<'a> {
    fn x(self, x: u16) -> Self;
    fn y(self, y: u16) -> Self;
    fn width(self, width: u16) -> Self;
    fn height(self, height: u16) -> Self;
}
pub trait ColouredWidget<'a>: SomeWidget<'a> {
    fn fg(self, fg: Option<Colour>) -> Self;
    fn bg(self, bg: Option<Colour>) -> Self;
}
pub trait TextualWidget<'a>: SomeWidget<'a> {
    fn text_vertical_align(self, align: TextVerticalAlign) -> Self;
    fn text_horizontal_align(self, align: TextHorizontalAlign) -> Self;
}
pub trait BorderedWidget<'a>: SomeWidget<'a> {
    fn border_style(self, style: Option<BorderStyle>) -> Self;
}

impl<'a, IE: IntoStatefulExecutable<'a, Args, State = WidgetState<State>>, Args, State>
    SomeWidget<'a> for Widget<'a, IE, Args, State>
{
    type Output = IE::Output;

    fn build(self) -> impl Executable<'a, Output = Self::Output> {
        self.executable.into_executable_with_state(self.state)
    }
}
impl<'a, IE: IntoStatefulExecutable<'a, Args, State = WidgetState<State>>, Args, State>
    PositionedWidget<'a> for Widget<'a, IE, Args, State>
{
    fn x(mut self, x: u16) -> Self {
        self.state.x = x;
        self
    }
    fn y(mut self, y: u16) -> Self {
        self.state.y = y;
        self
    }
    fn width(mut self, width: u16) -> Self {
        self.state.width = width;
        self
    }
    fn height(mut self, height: u16) -> Self {
        self.state.height = height;
        self
    }
}
impl<'a, IE: IntoStatefulExecutable<'a, Args, State = WidgetState<State>>, Args, State>
    ColouredWidget<'a> for Widget<'a, IE, Args, State>
{
    fn bg(mut self, bg: Option<Colour>) -> Self {
        self.state.bg = bg;
        self
    }
    fn fg(mut self, fg: Option<Colour>) -> Self {
        self.state.fg = fg;
        self
    }
}
impl<'a, IE: IntoStatefulExecutable<'a, Args, State = WidgetState<State>>, Args, State>
    TextualWidget<'a> for Widget<'a, IE, Args, State>
{
    fn text_vertical_align(mut self, align: TextVerticalAlign) -> Self {
        self.state.text_vertical_align = align;
        self
    }
    fn text_horizontal_align(mut self, align: TextHorizontalAlign) -> Self {
        self.state.text_horizontal_align = align;
        self
    }
}
impl<'a, IE: IntoStatefulExecutable<'a, Args, State = WidgetState<State>>, Args, State>
    BorderedWidget<'a> for Widget<'a, IE, Args, State>
{
    fn border_style(mut self, style: Option<BorderStyle>) -> Self {
        self.state.border_style = style;
        self
    }
}

#[cfg(test)]
mod tests {}
