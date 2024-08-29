pub mod input;
pub mod msg;
pub mod os;
pub mod runloop;
pub mod shapes;
pub mod terminal;
pub mod widgets;

pub mod prelude {
    pub use crate::{
        input::Key, msg::TuiMsg, runloop::TuiRunloop, shapes::*, terminal::Terminal, App, Colour,
        TuiPlugin,
    };
}

use {msg::TuiMsg, scaffolding::plugin_prelude::*, terminal::Terminal, widgets::SomeWidget};

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
    pub fn draw<'a, W: SomeWidget<'a>>(&self, widget: W) -> W::Output {
        widget.build().execute(self.0)
    }

    pub fn exit(&self) {
        self.0.send_msg(TuiMsg::ExitRunloop);
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
