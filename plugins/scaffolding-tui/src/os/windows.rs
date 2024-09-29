use {
    super::OsTrait,
    scaffolding::utils::bitflags,
    std::{ffi::c_void, mem::MaybeUninit, ptr},
};

// region: FFI

bitflags! {
    struct ConsoleModes: u32;
    bitflags ConsoleMode {
        EchoInput = 0x0004,
        InsertMode = 0x0020,
        LineInput = 0x0002,
        MouseInput = 0x0010,
        ProcessedInput = 0x0001,
        QuickEditMode = 0x0040,
        WindowInput = 0x0008,
        VirtualTerminalInput = 0x0200
    }
}

#[repr(C)]
struct Coord {
    x: i16,
    y: i16,
}
#[repr(C)]
struct SmallRect {
    left: i16,
    top: i16,
    right: i16,
    bottom: i16,
}
#[repr(C)]
struct ConsoleScreenBufferInfo {
    size: Coord,
    cursor_position: Coord,
    attributes: u16,
    window: SmallRect,
    maximum_window_size: Coord,
}

#[repr(transparent)]
#[derive(Clone, Copy)]
struct Handle(*mut c_void);
unsafe impl Sync for Handle {}
unsafe impl Send for Handle {}

#[repr(transparent)]
#[derive(Clone, Copy)]
struct Bool(i32);
impl Bool {
    fn as_bool(self) -> bool {
        self.0 != 0
    }
}

#[repr(u32)]
enum StdHandle {
    Input = -10i32 as u32,
    Output = -11i32 as u32,
}

#[derive(PartialEq, Eq)]
#[repr(u32)]
enum WaitForSingleObjectResult {
    Abandoned = 0x00000080,
    Object0 = 0x00000000,
    Timeout = 0x00000102,
    Failed = u32::MAX,
}

#[link(name = "kernel32")]
extern "C" {
    fn GetConsoleMode(hConsoleHandle: Handle, lpMode: *mut ConsoleModes) -> Bool;
    fn SetConsoleMode(hConsoleHandle: Handle, dwMode: ConsoleModes) -> Bool;
    fn GetConsoleScreenBufferInfo(
        hConsoleOutput: Handle,
        lpConsoleScreenBufferInfo: *mut ConsoleScreenBufferInfo,
    ) -> Bool;
    fn GetLastError() -> u32;
    fn GetStdHandle(nStdHandle: StdHandle) -> Handle;
    fn GetNumberOfConsoleInputEvents(hConsoleInput: Handle, lpcNumberOfEvents: *mut u32) -> Bool;
    fn WaitForSingleObject(hHandle: Handle, dwMilliseconds: u32) -> WaitForSingleObjectResult;
    fn ReadFile(
        hFile: Handle,
        lpBuffer: *mut c_void,
        lpNumberOfBytesToRead: u32,
        lpNumberOfBytesRead: *mut u32,
        lpOverlapped: *mut c_void,
    ) -> Bool;
}

// endregion

#[derive(Clone)]
pub struct Os {
    /// The original console mode, before we enable raw mode. We store it so we
    /// can reset to this mode later.
    og_console_mode: ConsoleModes,
    /// A handle for stdin.
    stdin_handle: Handle,
    /// A handle for stdout.
    stdout_handle: Handle,
}
impl Default for Os {
    fn default() -> Self {
        let stdin_handle = unsafe { GetStdHandle(StdHandle::Input) };
        let stdout_handle = unsafe { GetStdHandle(StdHandle::Output) };

        let mut og_console_mode = MaybeUninit::uninit();
        let res = unsafe { GetConsoleMode(stdin_handle, og_console_mode.as_mut_ptr()) };
        if !res.as_bool() {
            panic!(
                "scaffolding-tui: Failed to get current console's mode. Error code: {}",
                unsafe { GetLastError() }
            );
        }

        let og_console_mode = unsafe { og_console_mode.assume_init() };

        Self {
            og_console_mode,
            stdin_handle,
            stdout_handle,
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
                    ConsoleMode::MouseInput | ConsoleMode::VirtualTerminalInput,
                )
            };

            if !res.as_bool() {
                panic!("scaffolding-tui::os::windows::Os::set_raw_mode: SetConsoleMode call had an error. Error code: {}", unsafe { GetLastError() });
            }
        } else {
            let res = unsafe { SetConsoleMode(self.stdin_handle, self.og_console_mode) };

            if !res.as_bool() {
                panic!("scaffolding-tui::os::windows::Os::set_raw_mode: SetConsoleMode call had an error. Error code: {}", unsafe { GetLastError() });
            }
        }
    }
    fn read_stdin_no_block(&self, buffer: &mut Vec<u8>) {
        buffer.clear();

        let mut byte = 0u8;
        while unsafe { WaitForSingleObject(self.stdin_handle, 0) }
            == WaitForSingleObjectResult::Object0
        {
            println!("pre-read");
            let res = unsafe {
                ReadFile(
                    self.stdin_handle,
                    &mut byte as *mut u8 as *mut c_void,
                    1,
                    ptr::null_mut(),
                    ptr::null_mut(),
                )
            };
            if !res.as_bool() {
                panic!("scaffolding-tui::os::windows::Os::read_stdin_no_block: ReadFile call had an error. Error code: {}", unsafe { GetLastError() });
            }
            println!("post-read");

            buffer.push(byte);
        }
    }
}
