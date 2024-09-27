#[cfg(target_family = "unix")]
mod unix {
    use {
        libc::termios as Termios,
        std::{
            cell::Cell,
            io::{stdin, stdout},
            mem::MaybeUninit,
            os::fd::AsRawFd,
        },
    };

    #[repr(C)]
    #[derive(Default)]
    struct Winsize {
        row: u16,
        col: u16,
        xpixel: u16,
        ypixel: u16,
    }

    thread_local! {
        // The termios mode before we enabled raw mode, so we can reset to it
        // later.
        static OG_TERMIOS: Cell<Option<Termios>> = const { Cell::new(None) };
    }

    pub fn get_terminal_size() -> (u16, u16) {
        let mut size = Winsize::default();
        let res = unsafe {
            libc::ioctl(
                stdout().as_raw_fd(),
                libc::TIOCGWINSZ,
                &mut size as *mut Winsize,
            )
        };

        if res != 0 {
            eprintln!("WARN: ioctl call had an error");
        }

        (size.col - 1, size.row - 1)
    }
    pub fn set_raw_mode(enabled: bool) {
        if enabled {
            let mut termios = MaybeUninit::uninit();
            let mut termios = unsafe {
                libc::tcgetattr(stdin().as_raw_fd(), termios.as_mut_ptr());

                termios.assume_init()
            };

            OG_TERMIOS.with(|val| val.set(Some(termios)));

            unsafe {
                libc::cfmakeraw(&mut termios as *mut Termios);
                libc::tcsetattr(stdin().as_raw_fd(), libc::TCSAFLUSH, &termios);
            }
        } else if let Some(termios) = OG_TERMIOS.with(|termios| termios.get()) {
            unsafe {
                libc::tcsetattr(stdin().as_raw_fd(), libc::TCSAFLUSH, &termios);
            }
        }
    }
    pub fn set_blocking(blocking: bool) {
        // https://stackoverflow.com/a/68174244
        unsafe {
            let flags = libc::fcntl(stdin().as_raw_fd(), libc::F_GETFL);
            libc::fcntl(
                stdin().as_raw_fd(),
                libc::F_SETFL,
                if blocking {
                    flags & !libc::O_NONBLOCK
                } else {
                    flags | libc::O_NONBLOCK
                },
            );
        }
    }
}
#[cfg(target_family = "unix")]
pub use unix::*;

#[cfg(target_family = "windows")]
mod windows {
    use {
        scaffolding::utils::bitflags,
        std::{cell::Cell, ffi::c_void, mem::MaybeUninit},
    };

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
        }
    }

    thread_local! {
        // The console mode before we enabled raw mode, so we can reset to it
        // later.
        static OG_CONSOLE_MODE: Cell<Option<ConsoleModes>> = Cell::new(None);
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

    #[link(name = "kernel32")]
    extern "C" {
        fn GetConsoleMode(hConsoleHandle: *mut c_void, lpMode: *mut ConsoleModes) -> bool;
        fn SetConsoleMode(hConsoleHandle: *mut c_void, dwMode: ConsoleModes) -> bool;
        fn GetConsoleWindow() -> *mut c_void;
        fn GetConsoleScreenBufferInfo(
            hConsoleOutput: *mut c_void,
            lpConsoleScreenBufferInfo: *mut ConsoleScreenBufferInfo,
        ) -> bool;
    }

    pub fn get_terminal_size() -> (u16, u16) {
        let mut info = MaybeUninit::<ConsoleScreenBufferInfo>::uninit();
        let info = unsafe {
            GetConsoleScreenBufferInfo(
                GetConsoleWindow(),
                &mut info as *mut MaybeUninit<ConsoleScreenBufferInfo>
                    as *mut ConsoleScreenBufferInfo,
            );
            info.assume_init()
        };

        (
            info.size.x.try_into().unwrap(),
            info.size.y.try_into().unwrap(),
        )
    }
    pub fn set_raw_mode(enabled: bool) {
        if enabled {
            let mut og_mode = MaybeUninit::uninit();
            let og_mode = unsafe {
                GetConsoleMode(
                    GetConsoleWindow(),
                    &mut og_mode as *mut MaybeUninit<ConsoleModes> as *mut ConsoleModes,
                );
                og_mode.assume_init()
            };

            OG_CONSOLE_MODE.with(|mode| mode.set(Some(og_mode)));

            unsafe {
                SetConsoleMode(
                    GetConsoleWindow(),
                    ConsoleMode::MouseInput | ConsoleMode::VirtualTerminalInput,
                );
            }
        } else if let Some(mode) = OG_CONSOLE_MODE.with(|mode| mode.get()) {
            unsafe {
                SetConsoleMode(GetConsoleWindow(), mode);
            }
        }
    }
    pub fn set_blocking(blocking: bool) {}
}
#[cfg(target_family = "windows")]
pub use windows::*;
