#[cfg(target_family = "unix")]
mod unix {
    use {
        libc::termios as Termios,
        std::{
            cell::OnceCell,
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

    thread_local! {
        static OG_TERMIOS: OnceCell<Termios> = const { OnceCell::new() };
    }

    pub fn set_raw_mode(enabled: bool) {
        if enabled {
            let mut termios = MaybeUninit::uninit();
            unsafe {
                libc::tcgetattr(stdin().as_raw_fd(), termios.as_mut_ptr());

                let mut termios = termios.assume_init();
                OG_TERMIOS.with(|val| val.set(termios).unwrap_or_else(|_| panic!()));

                libc::cfmakeraw(&mut termios as *mut Termios);
                libc::tcsetattr(stdin().as_raw_fd(), libc::TCSAFLUSH, &termios);
            }
        } else {
            let termios = OG_TERMIOS.with(|termios| *termios.get().unwrap());
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
compile_error!("TODO: Windows terminal functions");
