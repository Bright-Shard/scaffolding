pub mod keys;
pub mod os;
pub mod runloop;
pub mod shapes;
pub mod terminal;

pub mod prelude {
    pub use crate::{
        keys::Key, runloop::TuiRunloop, shapes::*, terminal::Terminal, Colour,
        ScaffoldingTuiPlugin, TerminalMsgSender,
    };
}

use {scaffolding::plugin_prelude::*, terminal::Terminal};

#[derive(Default)]
pub struct ScaffoldingTuiPlugin {}
impl Plugin for ScaffoldingTuiPlugin {
    fn load(&mut self, world: &mut World) {
        world.add_singleton(Terminal::default());
        world.set_msg_handler(|world, _: MsgUpdateTerminal| {
            let terminal: &mut Terminal = world.get_singleton_mut();
            terminal.update()
        });
        world.set_msg_handler(|world, _: MsgExitTuiRunloop| {
            let terminal: &mut Terminal = world.get_singleton_mut();
            terminal.exit = true
        });
    }
}

pub trait TuiElement {
    type Output;

    fn draw(self, terminal: &Terminal) -> Self::Output;
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

/// A message to update the [`Terminal`] singleton.
pub struct MsgUpdateTerminal;

/// A message to stop the runloop started with [`runloop::TuiRunloop`].
pub struct MsgExitTuiRunloop;

#[derive(Default)]
pub struct TerminalMsgSender {
    update: bool,
    exit_tui_runloop: bool,
}
impl TerminalMsgSender {
    pub fn send_update(&mut self) {
        self.update = true
    }
    pub fn send_exit_tui_runloop(&mut self) {
        self.exit_tui_runloop = true
    }
}
impl ExecutableArg for TerminalMsgSender {
    type Arg<'a> = Self;

    fn build(_: &World) -> Self::Arg<'_> {
        Self::default()
    }

    fn on_drop(self) -> impl FnOnce(&mut World) + Send + 'static {
        move |world| {
            if self.update {
                world.send_msg(MsgUpdateTerminal);
            }

            if self.exit_tui_runloop {
                world.send_msg(MsgExitTuiRunloop);
            }
        }
    }
}
