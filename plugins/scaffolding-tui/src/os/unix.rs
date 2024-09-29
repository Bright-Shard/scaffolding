use {
    super::OsTrait,
    libc::{ioctl, termios as Termios, FIONREAD},
    std::{
        io::{stdin, stdout, ErrorKind, Read},
        mem::MaybeUninit,
        os::fd::{AsRawFd, RawFd},
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

#[derive(Clone)]
pub struct Os {
    /// Termios controls terminal settings. We store the termios of the terminal
    /// from before we enable/disable raw mode, so when we go back and disable
    /// raw mode, we can reset the terminal to its original settings.
    original_termios: Termios,
    /// The termios for enabling raw mode.
    raw_termios: Termios,
    /// The file descriptor for stdin.
    stdin: RawFd,
}
impl Default for Os {
    fn default() -> Self {
        let mut termios = MaybeUninit::uninit();
        let termios = unsafe {
            let status = libc::tcgetattr(stdin().as_raw_fd(), termios.as_mut_ptr());

            if status != 0 {
                panic!("scaffolding-tui: Failed to get current console's termios");
            }

            termios.assume_init()
        };

        let mut raw_termios = MaybeUninit::uninit();
        let raw_termios = unsafe {
            libc::cfmakeraw(raw_termios.as_mut_ptr());

            raw_termios.assume_init()
        };

        Self {
            original_termios: termios,
            raw_termios,
            stdin: stdin().as_raw_fd(),
        }
    }
}
impl OsTrait for Os {
    fn terminal_size(&self) -> (u16, u16) {
        let mut size = Winsize::default();
        let res = unsafe { libc::ioctl(self.stdin, libc::TIOCGWINSZ, &mut size as *mut Winsize) };

        if res != 0 {
            panic!("scaffolding-tui::os::unix::Os::terminal_size: ioctl call had an error");
        }

        (size.col - 1, size.row - 1)
    }
    fn set_raw_mode(&self, enabled: bool) {
        let termios = if enabled {
            &self.raw_termios
        } else {
            &self.original_termios
        };

        let res = unsafe { libc::tcsetattr(self.stdin, libc::TCSAFLUSH, termios) };

        if res != 0 {
            panic!("scaffolding-tui::os::unix::Os::set_raw_mode: tcsetattr call had an error");
        }
    }
    fn read_stdin_no_block(&self, buffer: &mut Vec<u8>) {
        buffer.clear();
        buffer.resize(10, 0);

        // https://stackoverflow.com/a/68174244
        let flags = unsafe { libc::fcntl(self.stdin, libc::F_GETFL) };
        let flags_nonblock = flags | libc::O_NONBLOCK;

        unsafe {
            libc::fcntl(self.stdin, libc::F_SETFL, flags_nonblock);
        }

        let mut bytes_read = 0;
        loop {
            match stdin().read(&mut buffer[bytes_read..]) {
                Ok(len) => {
                    bytes_read += len;
                    buffer.resize(buffer.len() * 2, 0);
                }
                Err(err) => match err.kind() {
                    ErrorKind::WouldBlock => {
                        break;
                    }
                    _ => panic!("Failed to read from stdin: {err}"),
                },
            }
        }

        buffer.truncate(bytes_read);

        unsafe {
            libc::fcntl(self.stdin, flags);
        }
    }
}
