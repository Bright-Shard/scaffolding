use {
    crate::{os, Colour, TuiElement},
    std::{
        io::{stdin, stdout, Read, Write},
        str,
        sync::Mutex,
    },
};

thread_local! {
    static TERM_SIZE: Mutex<(u32, u32)> = const { Mutex::new((0, 0)) };
}

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
    /// The buffer for reading from stdin.
    input_buffer: Vec<u8>,
}
impl Terminal {
    #[inline(always)]
    pub fn draw<E: TuiElement>(&self, element: E) -> E::Output {
        element.draw(self)
    }

    pub fn render_char(
        &self,
        figure: char,
        position: (u16, u16),
        fg: Option<Colour>,
        bg: Option<Colour>,
    ) {
        // Set fg colour
        if let Some(fg) = fg {
            print!("\x1B[38;2;{};{};{}m", fg.r, fg.g, fg.b);
        } else {
            // Default fg colour
            print!("\x1B[39m");
        }
        // Set bg colour
        if let Some(bg) = bg {
            print!("\x1B[48;2;{};{};{}m", bg.r, bg.g, bg.b);
        } else {
            // Default bg colour
            print!("\x1B[49m");
        }
        // Move cursor
        print!("\x1B[{};{}H", position.1 + 1, position.0 + 1);
        // Print figure
        print!("{figure}");
    }
    pub fn render_string(
        &self,
        string: &str,
        position: (u16, u16),
        fg: Option<Colour>,
        bg: Option<Colour>,
    ) {
        // Set fg colour
        if let Some(fg) = fg {
            print!("\x1B[38;2;{};{};{}m", fg.r, fg.g, fg.b);
        } else {
            // Default fg colour
            print!("\x1B[39m");
        }
        // Set bg colour
        if let Some(bg) = bg {
            print!("\x1B[48;2;{};{};{}m", bg.r, bg.g, bg.b);
        } else {
            // Default bg colour
            print!("\x1B[49m");
        }
        // Move cursor
        print!("\x1B[{};{}H", position.1 + 1, position.0 + 1);
        // Print string
        print!("{string}");
    }

    pub fn flush(&self) {
        stdout().flush().unwrap();
    }

    /// Resets styles/colours, clears the screen, and moves the cursor to
    /// (0, 0).
    pub fn reset(&self) {
        print!("\x1B[0m\x1B[2J\x1B[H");
        stdout().flush().unwrap();
    }

    pub fn update(&mut self) {
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
        let mut stdin = self.input_buffer.bytes();

        while let Some(byte) = stdin.next() {
            match byte.unwrap() {
                b'\x1B' => {
                    if stdin.next().unwrap().unwrap() == b'['
                        && stdin.next().unwrap().unwrap() == b'<'
                    {
                        // Format: \x1B[<, then mouse btn, then ;, then mouse x,
                        // then ;, then mouse y, then M or m for clicked/not
                        // clicked
                        let mut btn = 0;
                        while let Some(Ok(byte)) = stdin.next() {
                            if byte == b';' {
                                break;
                            }

                            btn *= 10;
                            btn += byte as u16 - 48;
                        }
                        let mut x = 0;
                        while let Some(Ok(byte)) = stdin.next() {
                            if byte == b';' {
                                break;
                            }

                            x *= 10;
                            x += byte as u16 - 48;
                        }

                        let mut y = 0;
                        let mut clicked = false;
                        while let Some(Ok(byte)) = stdin.next() {
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
                    } else {
                        panic!();
                    }
                }
                other => {
                    println!("Got char {}", other as char);
                }
            }
        }
    }
}
impl Default for Terminal {
    fn default() -> Self {
        os::set_raw_mode(true);
        os::set_blocking(false);

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
            input_buffer: Vec::with_capacity(10),
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
