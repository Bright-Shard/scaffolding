use {
    super::OsTrait,
    crate::{input::*, prelude::Terminal},
    libc::termios as Termios,
    std::{
        io::{stdin, ErrorKind, Read},
        mem::MaybeUninit,
        os::fd::{AsRawFd, RawFd},
        str,
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
    /// A buffer for reading text input from stdin.
    input_buffer: Vec<u8>,
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
            input_buffer: Vec::new(),
        }
    }
}
impl Os {
    /// Read from stdin without blocking the current thread.
    ///
    /// Normally, reading from stdin when it's empty causes the thread to block
    /// until bytes are added to stdin. For a TUI app, this is bad behaviour,
    /// because it will cause the app to freeze when the user isn't actively
    /// typing/moving their mouse.
    ///
    /// This method will clear `buffer`, then write the bytes from stdin (if
    /// there are any) to `buffer` afterwards.
    fn read_stdin_no_block(&mut self) {
        self.input_buffer.clear();
        self.input_buffer.resize(10, 0);

        // https://stackoverflow.com/a/68174244
        let flags = unsafe { libc::fcntl(self.stdin, libc::F_GETFL) };
        let flags_nonblock = flags | libc::O_NONBLOCK;

        unsafe {
            libc::fcntl(self.stdin, libc::F_SETFL, flags_nonblock);
        }

        let mut bytes_read = 0;
        loop {
            match stdin().read(&mut self.input_buffer[bytes_read..]) {
                Ok(len) => {
                    bytes_read += len;
                    self.input_buffer.resize(self.input_buffer.len() * 2, 0);
                }
                Err(err) => match err.kind() {
                    ErrorKind::WouldBlock => {
                        break;
                    }
                    _ => panic!("Failed to read from stdin: {err}"),
                },
            }
        }

        self.input_buffer.truncate(bytes_read);

        unsafe {
            libc::fcntl(self.stdin, flags);
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
    fn update(terminal: &mut Terminal) {
        terminal.os.read_stdin_no_block();
        let mut stdin = terminal
            .os
            .input_buffer
            .iter()
            .copied()
            .enumerate()
            .peekable();

        while let Some((idx, byte)) = stdin.next() {
            match byte {
                b'\x1B' => {
                    // Escape sequence: This is either reporting a mouse
                    // movement or a special character
                    let next = stdin.next();
                    if matches!(next, Some((_, b'['))) {
                        let Some((_, next)) = stdin.next() else {
                            eprintln!("WARN: Received incomplete escape code from terminal");
                            continue;
                        };

                        match next {
                            // Mouse event
                            b'<' => {
                                // Format: \x1B[<, then mouse btn, then ;, then
                                // mouse x, then ;, then mouse y, then M or m
                                // for clicked/not clicked
                                let mut btn = 0;
                                for (_, byte) in &mut stdin {
                                    if byte == b';' {
                                        break;
                                    }

                                    btn *= 10;
                                    btn += byte as u16 - 48;
                                }
                                let mut x = 0;
                                for (_, byte) in &mut stdin {
                                    if byte == b';' {
                                        break;
                                    }

                                    x *= 10;
                                    x += byte as u16 - 48;
                                }

                                let mut y = 0;
                                let mut clicked = false;
                                for (_, byte) in &mut stdin {
                                    if byte == b'm' {
                                        break;
                                    } else if byte == b'M' {
                                        clicked = true;
                                        break;
                                    }

                                    y *= 10;
                                    y += byte as u16 - 48;
                                }

                                // Mouse bits:
                                // lowest 2 indicate mouse buttons 1-3
                                // next 3 are modifiers shift, meta, and control
                                // next bit indicates mouse motion
                                // next bit is mouse buttons 4-7 (4 and 5 mean
                                // scroll)
                                // next bit is mouse buttons 8-11
                                let mut button_number = btn & 0b0000_0011;

                                if btn & 0b0100_0000 != 0 {
                                    // bit for 4-7 range
                                    button_number += 3;
                                } else if btn & 0b1000_0000 != 0 {
                                    // bit for 8-11 range
                                    button_number += 7;
                                }

                                // modifier bits
                                terminal.modifier_keys.shift = (btn & 0b0000_0100) != 0;
                                terminal.modifier_keys.meta = (btn & 0b0000_1000) != 0;
                                terminal.modifier_keys.control = (btn & 0b0001_0000) != 0;

                                if button_number == 4 {
                                    terminal.scroll_direction = Some(ScrollDirection::Backwards);
                                } else if button_number == 5 {
                                    terminal.scroll_direction = Some(ScrollDirection::Forwards);
                                } else {
                                    // -1 cause it starts indexing pixels at 1
                                    terminal.mouse_pos = (x - 1, y - 1);
                                    let btn = button_number as u8;
                                    if clicked {
                                        if !terminal.held_mouse_buttons.contains(&btn) {
                                            terminal.clicked_mouse_buttons.insert(btn);
                                        }
                                    } else {
                                        terminal.clicked_mouse_buttons.remove(&btn);
                                        terminal.held_mouse_buttons.remove(&btn);
                                        terminal.released_mouse_buttons.insert(btn);
                                    }
                                }
                            }

                            // Arrow keys
                            b'A' => {
                                terminal.pressed_keys.insert(Key::ArrowUp);
                            }
                            b'B' => {
                                terminal.pressed_keys.insert(Key::ArrowDown);
                            }
                            b'C' => {
                                terminal.pressed_keys.insert(Key::ArrowRight);
                            }
                            b'D' => {
                                terminal.pressed_keys.insert(Key::ArrowLeft);
                            }

                            // Group of special keys that end with ~
                            other if stdin.next().map(|(_, byte)| byte) == Some(b'~') => {
                                match other {
                                    b'5' => {
                                        terminal.pressed_keys.insert(Key::PageUp);
                                    }
                                    b'6' => {
                                        terminal.pressed_keys.insert(Key::PageDown);
                                    }
                                    b'1' | b'7' => {
                                        terminal.pressed_keys.insert(Key::Home);
                                    }
                                    b'4' | b'8' => {
                                        terminal.pressed_keys.insert(Key::End);
                                    }
                                    b'3' => {
                                        terminal.pressed_keys.insert(Key::Delete);
                                    }
                                    _ => eprintln!(
                                        "WARN: Unknown special key escape sequence: ESC[{}~",
                                        other as char
                                    ),
                                }
                            }

                            // Home and end (note they can also be sent in the
                            // group above)
                            b'H' => {
                                terminal.pressed_keys.insert(Key::Home);
                            }
                            b'F' => {
                                terminal.pressed_keys.insert(Key::End);
                            }
                            b'O' => {
                                let Some((_, next)) = stdin.next() else {
                                    println!("WARN: Got incomplete control key sequence ESC[O");
                                    continue;
                                };
                                match next {
                                    b'H' => {
                                        terminal.pressed_keys.insert(Key::Home);
                                    }
                                    b'F' => {
                                        terminal.pressed_keys.insert(Key::End);
                                    }
                                    _ => println!(
                                        "WARN: Unknown special key escape sequence: ESC[O{}",
                                        next as char
                                    ),
                                }
                            }

                            _ => {}
                        }
                    } else if next.is_none() {
                        terminal.pressed_keys.insert(Key::Escape);
                    }
                }
                _ => {
                    // Non-escape sequences: we're receiving a normal key
                    // This is either ASCII (k, a) or a string from IME (か).

                    // We could be receiving multiple keyboard events in this
                    // update call, so we need to make sure we don't read an
                    // escape character
                    let mut len = 1;
                    while !matches!(stdin.peek().copied(), Some((_, b'\x1B')))
                        && stdin.peek().is_some()
                    {
                        stdin.next().unwrap();
                        len += 1;
                    }

                    // Convert whatever we received to UTF-8
                    let Ok(text) = str::from_utf8(&terminal.os.input_buffer[idx..idx + len]) else {
                        eprintln!("WARN: Got invalid UTF-8 from the terminal");
                        continue;
                    };
                    for char in text.chars() {
                        if char == '\x7F' {
                            terminal.pressed_keys.insert(Key::Backspace);
                        } else {
                            terminal.pressed_keys.insert(Key::Text(char));
                        }
                    }
                }
            }
        }
    }
}
