pub mod input;
pub mod msg;
pub mod os;
pub mod runloop;
pub mod shapes;
pub mod terminal;
pub mod widgets;

pub mod prelude {
    pub use crate::{
        input::Key,
        msg::TuiMsg,
        runloop::TuiRunloop,
        shapes::*,
        terminal::Terminal,
        widgets::{Button, ButtonState, Checkbox, Frame, HAlign, Text, TextInput, VAlign},
        App, Colour, TuiPlugin,
    };
}

use {msg::TuiMsg, scaffolding::plugin_prelude::*, terminal::Terminal, widgets::Widget};

#[derive(Default)]
pub struct TuiPlugin {}
impl Plugin for TuiPlugin {
    fn load(&mut self, world: &mut World) {
        world
            .add_singleton(Terminal::default())
            .add_msg_handler(msg::tui_msg_handler);
    }
}

pub struct App<'a>(&'a World);
impl ExecutableArg for App<'_> {
    type Arg<'a> = App<'a>;

    fn build(world: &World) -> Self::Arg<'_> {
        App(world)
    }
    fn drop(self, _: &World) {}
}
impl App<'_> {
    pub fn draw<'a, Args, D: Drawable<'a, Args>>(&self, drawable: D) -> D::Output {
        drawable.build().execute(self.0)
    }

    pub fn exit(&self) {
        self.0.send_msg(TuiMsg::ExitRunloop);
    }
}

/// Types that can be used with [`App::draw`]. This is implemented for
/// [`Widget`]s and [`Executable`]s by default.
pub trait Drawable<'a, Args> {
    type Output: 'a;
    fn build(self) -> impl TypeErasedExecutable<'a, Output = Self::Output>;
}
impl<'a, W: Widget<'a>> Drawable<'a, ()> for W {
    type Output = W::Output;

    fn build(self) -> impl TypeErasedExecutable<'a, Output = Self::Output> {
        self.build_draw_fn()
    }
}
impl<'a, Args: 'a, E: Executable<'a, Args>> Drawable<'a, Option<Args>> for E {
    type Output = E::Output;

    fn build(self) -> impl TypeErasedExecutable<'a, Output = Self::Output> {
        self.type_erase()
    }
}

#[derive(Clone, Copy)]
pub struct Colour {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}
impl Colour {
    pub const BLACK: Self = Self::new(0, 0, 0);
    pub const WHITE: Self = Self::new(255, 255, 255);
    pub const RED: Self = Self::new(255, 0, 0);
    pub const GREEN: Self = Self::new(0, 255, 0);
    pub const BLUE: Self = Self::new(0, 0, 255);
    pub const GREY: Self = Self::new(127, 127, 127);

    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}
