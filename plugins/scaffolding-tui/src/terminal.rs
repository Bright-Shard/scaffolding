use {
    crate::{keys::Key, os, Colour, TuiElement},
    scaffolding::{datatypes::ArenaVec, utils::MemoryAmount},
    std::{
        collections::HashSet,
        fmt::Write as _,
        io::{stdin, stdout, Read, Write as _},
        str,
    },
};

/// Handles communicating with the terminal using ANSI escape sequences to
/// query input and render the TUI.
pub struct Terminal {
    /// The width and height of the terminal we're rendering in.
    pub size: (u16, u16),
    /// The current location of the mouse.
    pub mouse_pos: (u16, u16),
    /// Any actively held modifier keys.
    pub mouse_modifiers: MouseModifiers,
    /// What the mouse is currently doing.
    pub mouse_state: MouseState,
    /// Keys currently held by the user.
    pub pressed_keys: HashSet<Key>,
    /// If we should exit the app.
    pub exit: bool,
    /// The buffer for reading from stdin.
    input_buffer: Vec<u8>,
    /// The buffer for writing to stdout.
    output_buffer: ArenaVec<u8>,
}
impl Terminal {
    #[inline(always)]
    pub fn draw<E: TuiElement>(&self, element: E) -> E::Output {
        element.draw(self)
    }

    pub fn render_bytes(
        &self,
        bytes: &[u8],
        position: (u16, u16),
        fg: Option<Colour>,
        bg: Option<Colour>,
    ) {
        let mut buffer = &self.output_buffer;

        // Set fg colour
        if let Some(fg) = fg {
            write!(buffer, "\x1B[38;2;{};{};{}m", fg.r, fg.g, fg.b).unwrap();
        } else {
            // Default fg colour
            buffer.extend_from_slice(b"\x1B[39m");
        }
        // Set bg colour
        if let Some(bg) = bg {
            write!(buffer, "\x1B[48;2;{};{};{}m", bg.r, bg.g, bg.b).unwrap();
        }
        // Move cursor
        write!(buffer, "\x1B[{};{}H", position.1 + 1, position.0 + 1).unwrap();
        // Print bytes
        buffer.extend_from_slice(bytes);
    }
    pub fn render_char(
        &self,
        figure: char,
        position: (u16, u16),
        fg: Option<Colour>,
        bg: Option<Colour>,
    ) {
        let mut buf = [0; 4];
        let string = figure.encode_utf8(&mut buf);
        self.render_bytes(string.as_bytes(), position, fg, bg)
    }
    pub fn render_string(
        &self,
        string: &str,
        position: (u16, u16),
        fg: Option<Colour>,
        bg: Option<Colour>,
    ) {
        self.render_bytes(string.as_bytes(), position, fg, bg)
    }

    pub fn update(&mut self) {
        print!("\x1B[0m\x1B[2J\x1B[H");
        stdout().flush().unwrap();
        stdout().write_all(&self.output_buffer).unwrap();
        stdout().flush().unwrap();
        self.output_buffer.clear();

        os::set_blocking(false);

        // Get terminal size
        self.size = os::get_terminal_size();

        // Handle user input
        self.input_buffer.resize(10, 0);
        let mut bytes_read = 0;
        loop {
            if let Ok(new_bytes_read) = stdin().read(&mut self.input_buffer[bytes_read..]) {
                bytes_read += new_bytes_read;
            } else {
                break;
            }

            if bytes_read < self.input_buffer.len() {
                break;
            }

            self.input_buffer.reserve(self.input_buffer.len());
            for _ in 0..self.input_buffer.len() {
                self.input_buffer.push(0);
            }
        }

        self.input_buffer.truncate(bytes_read);
        let mut stdin = self.input_buffer.iter().copied().enumerate().peekable();

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
                                // Format: \x1B[<, then mouse btn, then ;, then mouse x,
                                // then ;, then mouse y, then M or m for clicked/not
                                // clicked
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
                                // next bit is mouse buttons 4-7 (4 and 5 mean scroll)
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
                                self.mouse_modifiers.shift = (btn & 0b0000_0100) != 0;
                                self.mouse_modifiers.meta = (btn & 0b0000_1000) != 0;
                                self.mouse_modifiers.control = (btn & 0b0001_0000) != 0;

                                if button_number == 4 {
                                    self.mouse_state = MouseState::ScrollBack;
                                } else if button_number == 5 {
                                    self.mouse_state = MouseState::ScrollForward;
                                } else {
                                    // -1 cause it starts indexing pixels at 1
                                    self.mouse_pos = (x - 1, y - 1);
                                    self.mouse_state = if clicked {
                                        MouseState::Pressed(button_number as _)
                                    } else {
                                        MouseState::Released
                                    };
                                }
                            }

                            // Arrow keys
                            b'A' => {
                                println!("Got arrow up")
                            }
                            b'B' => {
                                println!("Got arrow down")
                            }
                            b'C' => {
                                println!("Got arrow right")
                            }
                            b'D' => {
                                println!("Got arrow left")
                            }

                            // Group of special keys that end with ~
                            other if stdin.next().map(|(_, byte)| byte) == Some(b'~') => {
                                match other {
                                    b'5' => println!("Got page up"),
                                    b'6' => println!("Got page down"),
                                    b'1' | b'7' => println!("Got home key"),
                                    b'4' | b'8' => println!("Got end key"),
                                    _ => eprintln!(
                                        "WARN: Unknown special key escape sequence: ESC[{}~",
                                        other as char
                                    ),
                                }
                            }

                            // Home and end (note they can also be sent in the
                            // group above)
                            b'H' => println!("Got home key"),
                            b'F' => println!("Got end key"),
                            b'O' => {
                                let Some((_, next)) = stdin.next() else {
                                    println!("WARN: Got incomplete control key sequence ESC[O");
                                    continue;
                                };
                                match next {
                                    b'H' => println!("Got home key"),
                                    b'F' => println!("Got end key"),
                                    _ => println!(
                                        "WARN: Unknown special key escape sequence: ESC[O{}",
                                        next as char
                                    ),
                                }
                            }

                            _ => {}
                        }
                    } else if next.is_none() {
                        self.pressed_keys.insert(Key::Escape);
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
                    let Ok(text) = str::from_utf8(&self.input_buffer[idx..idx + len]) else {
                        eprintln!("WARN: Got invalid UTF-8 from the terminal");
                        continue;
                    };
                    println!("Got char {}", text);
                }
            }
        }

        os::set_blocking(true);
    }
}
impl Default for Terminal {
    fn default() -> Self {
        os::set_raw_mode(true);

        const INITIAL_COMMANDS: &str = concat!(
            // UTF-8 character set
            "\x1B[%G",
            // ===
            // below are settings that should be reset in [`FINAL_COMMANDS`]
            // ===
            // hide the cursor
            "\x1B[?25l",
            // enter the alternate buffer
            // this is an alternate screen that doesn't scrollback, so we can
            // just draw to it and won't be deleting terminal history
            "\x1B[?1049h",
            // enable mouse location reporting
            "\x1B[?1003h",
            // enable SGR extended mouse location reporting
            // without this, mouse x/y coords are each limited between 0 and 223
            "\x1B[?1006h",
        );
        stdout().write_all(INITIAL_COMMANDS.as_bytes()).unwrap();
        stdout().flush().unwrap();

        Self {
            size: (0, 0),
            mouse_pos: (0, 0),
            mouse_modifiers: MouseModifiers::default(),
            mouse_state: MouseState::Released,
            pressed_keys: HashSet::default(),
            exit: false,
            input_buffer: Vec::with_capacity(10),
            output_buffer: ArenaVec::with_reserved_memory(MemoryAmount::Megabytes(1).into_bytes()),
        }
    }
}
impl Drop for Terminal {
    fn drop(&mut self) {
        // disable all of the things we enabled in [`INITIAL_COMMANDS`]
        const FINAL_COMMANDS: &str = concat!(
            // show the cursor
            "\x1B[?25h",
            // leave the alternate buffer
            "\x1B[?1049l",
            // disable mouse location reporting
            "\x1B[?1003l",
            // disable SGR extended mouse location reporting
            "\x1B[?1006l",
        );
        stdout().write_all(FINAL_COMMANDS.as_bytes()).unwrap();
        stdout().flush().unwrap();

        os::set_raw_mode(false);
    }
}

#[derive(Default)]
pub struct MouseModifiers {
    pub shift: bool,
    pub meta: bool,
    pub control: bool,
}
pub enum MouseState {
    Pressed(u8),
    Released,
    /// Scroll so text that was previously off the top of the screen is now
    /// visible.
    ScrollBack,
    /// Scroll so text that was previously off the bottom of the screen is now
    /// visible.
    ScrollForward,
}
