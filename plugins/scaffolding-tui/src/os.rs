//! OS APIs to interact with the terminal.

use crate::terminal::Terminal;

pub trait OsTrait: Default + Clone {
    /// Get the terminal's size, in rows and columns.
    fn terminal_size(&self) -> (u16, u16);
    /// Toggle raw mode.
    ///
    /// In raw mode, the terminal will report key events to us immediately,
    /// instead of when the user hits enter.
    fn set_raw_mode(&self, enabled: bool);
    /// Read input from the user and update the terminal's state. This updates
    /// the mouse location, pressed keys, etc.
    fn update(terminal: &mut Terminal);
}

#[cfg_attr(target_family = "unix", path = "os/unix.rs")]
#[cfg_attr(target_family = "windows", path = "os/windows.rs")]
mod os_impl;

pub use os_impl::Os;
