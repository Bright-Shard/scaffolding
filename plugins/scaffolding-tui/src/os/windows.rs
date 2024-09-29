use {
    super::OsTrait,
    crate::terminal::Terminal,
    std::{mem::MaybeUninit, ptr},
};

#[path = "windows/ffi.rs"]
mod ffi;
use ffi::*;

#[derive(Clone)]
pub struct Os {
    /// The original console mode for stdin, before we enable raw mode. We
    /// store it so we can reset to this mode later.
    stdin_og_mode: ConsoleModes,
    /// Same as [`Os::stdin_og_mode`], but for stdout.
    stdout_og_mode: ConsoleModes,
    /// A handle for stdin.
    stdin_handle: Handle,
    /// A handle for stdout.
    stdout_handle: Handle,
    /// A buffer for reading from the console.
    input_buffer: Vec<InputRecord>,
}
impl Default for Os {
    fn default() -> Self {
        let stdin_handle = unsafe { GetStdHandle(StdHandle::Input) };
        let stdout_handle = unsafe { GetStdHandle(StdHandle::Output) };

        let mut stdin_og_mode = MaybeUninit::uninit();
        let res = unsafe { GetConsoleMode(stdin_handle, stdin_og_mode.as_mut_ptr()) };
        if !res.as_bool() {
            panic!(
                "scaffolding-tui: Failed to get current console's stdout mode. Error code: {}",
                unsafe { GetLastError() }
            );
        }
        let stdin_og_mode = unsafe { stdin_og_mode.assume_init() };

        let mut stdout_og_mode = MaybeUninit::uninit();
        let res = unsafe { GetConsoleMode(stdout_handle, stdout_og_mode.as_mut_ptr()) };
        if !res.as_bool() {
            panic!(
                "scaffolding-tui: Failed to get current console's stdin mode. Error code: {}",
                unsafe { GetLastError() }
            );
        }
        let stdout_og_mode = unsafe { stdout_og_mode.assume_init() };

        Self {
            stdin_og_mode,
            stdout_og_mode,
            stdin_handle,
            stdout_handle,
            input_buffer: Vec::default(),
        }
    }
}
impl Os {
    fn read_input(&mut self) {
        let mut num_events = 0u32;
        let res = unsafe { GetNumberOfConsoleInputEvents(self.stdin_handle, &mut num_events) };

        if !res.as_bool() {
            panic!("scaffolding-tui::os::windows::Os::update: GetNumberOfConsoleInputEvents call had an error. Error code: {}", unsafe { GetLastError() });
        }

        self.input_buffer.clear();

        if num_events > 0 {
            self.input_buffer.reserve(num_events as usize);

            let mut events_read = 0u32;
            let res = unsafe {
                ReadConsoleInputW(
                    self.stdin_handle,
                    self.input_buffer.as_mut_ptr(),
                    num_events,
                    &mut events_read,
                )
            };

            if !res.as_bool() {
                panic!("scaffolding-tui::os::windows::Os::update: ReadConsoleInput call had an error. Error code: {}", unsafe { GetLastError() });
            }

            unsafe { self.input_buffer.set_len(events_read as usize) };
        }
    }
}
impl OsTrait for Os {
    fn terminal_size(&self) -> (u16, u16) {
        let mut info = MaybeUninit::uninit();
        let res = unsafe { GetConsoleScreenBufferInfo(self.stdout_handle, info.as_mut_ptr()) };

        if !res.as_bool() {
            panic!("scaffolding-tui::os::windows::Os::terminal_size: GetConsoleScreenBufferInfo call had an error. Error code: {}", unsafe { GetLastError() });
        }

        let info = unsafe { info.assume_init() };

        (
            info.size.x.try_into().unwrap(),
            info.size.y.try_into().unwrap(),
        )
    }
    fn set_raw_mode(&self, enabled: bool) {
        if enabled {
            let res = unsafe {
                SetConsoleMode(
                    self.stdin_handle,
                    ConsoleMode::MouseInput
                        | ConsoleMode::ProcessedInput
                        | ConsoleMode::WindowInput
                        // This isn't documented, but mouse input breaks without
                        // this flag.
                        | ConsoleMode::ExtendedFlags,
                )
            };
            if !res.as_bool() {
                panic!("scaffolding-tui::os::windows::Os::set_raw_mode: SetConsoleMode on stdin call had an error. Error code: {}", unsafe { GetLastError() });
            }

            let res = unsafe {
                SetConsoleMode(
                    self.stdout_handle,
                    // means ProcessedOutput for stdout
                    ConsoleMode::ProcessedInput
                        // means VirtualTerminalProcessing for stdout
                        | ConsoleMode::EchoInput,
                )
            };
            if !res.as_bool() {
                panic!("scaffolding-tui::os::windows::Os::set_raw_mode: SetConsoleMode on stdin call had an error. Error code: {}", unsafe { GetLastError() });
            }
        } else {
            let res = unsafe { SetConsoleMode(self.stdin_handle, self.stdin_og_mode) };
            if !res.as_bool() {
                panic!("scaffolding-tui::os::windows::Os::set_raw_mode: SetConsoleMode call had an error. Error code: {}", unsafe { GetLastError() });
            }
            let res = unsafe { SetConsoleMode(self.stdout_handle, self.stdout_og_mode) };
            if !res.as_bool() {
                panic!("scaffolding-tui::os::windows::Os::set_raw_mode: SetConsoleMode call had an error. Error code: {}", unsafe { GetLastError() });
            }
        }
    }
    fn update(terminal: &mut Terminal) {
        terminal.os.read_input();

        for input in terminal.os.input_buffer.drain(..) {
            match input.event_type {
                EventType::Key => {
                    let key_event = unsafe { input.event.key_event };
                }
                EventType::Mouse => {
                    let mouse_event = unsafe { input.event.mouse_event };
                    terminal.mouse_pos = (
                        mouse_event.mouse_position.x.try_into().unwrap(),
                        mouse_event.mouse_position.y.try_into().unwrap(),
                    );
                }
                _ => {}
            }
        }
    }
}
