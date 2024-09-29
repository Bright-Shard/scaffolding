use {scaffolding::utils::bitflags, std::ffi::c_void};

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
        VirtualTerminalInput = 0x0200,
        // Not documented but appears to be correct:  https://github.com/search?q=repo%3Amicrosoft%2Fwindows-rs%20ENABLE_EXTENDED_FLAGS&type=code
        ExtendedFlags = 128
    }
}

#[repr(C)]
pub struct SmallRect {
    pub left: i16,
    pub top: i16,
    pub right: i16,
    pub bottom: i16,
}
#[repr(C)]
pub struct ConsoleScreenBufferInfo {
    pub size: Coord,
    pub cursor_position: Coord,
    pub attributes: u16,
    pub window: SmallRect,
    pub maximum_window_size: Coord,
}

#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct Handle(*mut c_void);
unsafe impl Sync for Handle {}
unsafe impl Send for Handle {}

#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct Bool(i32);
impl Bool {
    pub fn as_bool(self) -> bool {
        self.0 != 0
    }
}

#[repr(u32)]
pub enum StdHandle {
    Input = -10i32 as u32,
    Output = -11i32 as u32,
}

#[repr(u32)]
#[derive(Clone, Copy)]
pub enum MouseEventFlags {
    PressOrRelease = 0,
    DoubleClick = 0x0002,
    MouseHWheeled = 0x0008,
    MouseMoved = 0x0001,
    MouseWheeled = 0x0004,
}
bitflags! {
    struct MouseButtons: u32;
    bitflags MouseButton {
        Left = 0x0001,
        Right = 0x0002,
        Button3 = 0x0004,
        Button4 = 0x0008,
        Button5 = 0x0010,
    }
}
bitflags! {
    struct ControlKeys: u32;
    bitflags ControlKey {
        CapsLock = 0x0080,
        Enhanced = 0x0100,
        LeftAlt = 0x0002,
        LeftCtrl = 0x0008,
        Numlock = 0x0020,
        RightAlt = 0x0001,
        RightCtrl = 0x0004,
        ScrollLock = 0x0040,
        Shift = 0x0010,
    }
}

#[derive(Clone, Copy)]
#[repr(C)]
pub union UChar {
    pub unicode_char: u16,
    pub ascii_char: u8,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Coord {
    pub x: i16,
    pub y: i16,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct FocusEventRecord {
    pub set_focus: Bool,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct KeyEventRecord {
    pub key_down: Bool,
    pub repeat_count: u16,
    pub virtual_key_code: u16,
    pub virtual_scan_code: u16,
    pub char: UChar,
    pub control_key_state: u32,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct MenuEventRecord {
    pub command_id: u32,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct MouseEventRecord {
    pub mouse_position: Coord,
    pub button_state: MouseButtons,
    pub control_key_state: u32,
    pub event_flags: MouseEventFlags,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct WindowBufferSizeRecord {
    pub size: Coord,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub union Event {
    pub key_event: KeyEventRecord,
    pub mouse_event: MouseEventRecord,
    pub window_buffer_size: WindowBufferSizeRecord,
    pub menu_event: MenuEventRecord,
    pub focus_event: FocusEventRecord,
}
#[repr(u16)]
#[derive(Clone, Copy, Debug)]
pub enum EventType {
    Focus = 0x0010,
    Key = 0x0001,
    Menu = 0x0008,
    Mouse = 0x0002,
    WindowBufferSize = 0x0004,
}

#[repr(C)]
#[derive(Clone)]
pub struct InputRecord {
    pub event_type: EventType,
    pub event: Event,
}

#[link(name = "kernel32")]
extern "C" {
    pub fn GetConsoleMode(hConsoleHandle: Handle, lpMode: *mut ConsoleModes) -> Bool;
    pub fn SetConsoleMode(hConsoleHandle: Handle, dwMode: ConsoleModes) -> Bool;
    pub fn GetConsoleScreenBufferInfo(
        hConsoleOutput: Handle,
        lpConsoleScreenBufferInfo: *mut ConsoleScreenBufferInfo,
    ) -> Bool;
    pub fn GetLastError() -> u32;
    pub fn GetStdHandle(nStdHandle: StdHandle) -> Handle;
    pub fn GetNumberOfConsoleInputEvents(
        hConsoleInput: Handle,
        lpcNumberOfEvents: *mut u32,
    ) -> Bool;
    pub fn ReadConsoleInputW(
        hConsoleInput: Handle,
        lpBuffer: *mut InputRecord,
        nLength: u32,
        lpNumberOfEventsRead: *mut u32,
    ) -> Bool;
}
