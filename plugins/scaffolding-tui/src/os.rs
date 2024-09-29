//! OS APIs to interact with the terminal.

pub trait OsTrait: Default + Clone {
    /// Get the terminal's size, in rows and columns.
    fn terminal_size(&self) -> (u16, u16);
    /// Toggle raw mode.
    ///
    /// In raw mode, the terminal will report key events to us immediately,
    /// instead of when the user hits enter.
    fn set_raw_mode(&self, enabled: bool);
    /// Read from stdin without blocking the current thread.
    ///
    /// Normally, reading from stdin when it's empty causes the thread to block
    /// until bytes are added to stdin. For a TUI app, this is bad behaviour,
    /// because it will cause the app to freeze when the user isn't actively
    /// typing/moving their mouse.
    ///
    /// This method will clear `buffer`, then write the bytes from stdin (if
    /// there are any) to `buffer` afterwards.
    fn read_stdin_no_block(&self, buffer: &mut Vec<u8>);
}

#[cfg_attr(target_family = "unix", path = "os/unix.rs")]
#[cfg_attr(target_family = "windows", path = "os/windows.rs")]
mod os_impl;

pub use os_impl::Os;
