use {
    crate::terminal::Terminal,
    scaffolding::world::{Msg, World},
};

pub enum TuiMsg {
    /// If the app is running in a [`TuiRunloop`], this causes that runloop
    /// to exit.
    ///
    /// [`TuiRunloop`]: crate::runloop::TuiRunloop
    ExitRunloop,
    /// Redraws the UI and updates user input. If the app is running in a
    /// [`TuiRunloop`], this message is automatically sent for you every frame.
    ///
    /// [`TuiRunloop`]: crate::runloop::TuiRunloop
    UpdateTerminal,
}

pub fn tui_msg_handler(world: &mut World, msg: Msg<TuiMsg>) {
    let terminal: &mut Terminal = world.get_singleton_mut();

    match msg.read() {
        TuiMsg::ExitRunloop => terminal.exit = true,
        TuiMsg::UpdateTerminal => terminal.update(),
    }
}
