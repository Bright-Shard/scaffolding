use {
    crate::{input::*, os, shapes::Shape, Colour},
    scaffolding::{datatypes::ArenaVec, utils::MemoryAmount},
    std::{
        cell::Cell,
        collections::HashSet,
        fmt::Write as _,
        io::{stdin, stdout, Read, Write as _},
        str,
        sync::atomic::{AtomicBool, Ordering},
    },
};

/// Tracks if a [`Terminal`] was already dropped. When dropped, the [`Terminal`]
/// issues several commands to the terminal emulator to "reset" it to its normal
/// state (See [`Terminal::on_drop`] for more info). Running this code twice,
/// however, can cause weird bugs in the terminal emulator, so we first check
/// this boolean to make sure the drop code is only run once.
static TERMINAL_DROPPED: AtomicBool = AtomicBool::new(false);

/// Handles communicating with the terminal using ANSI escape sequences to
/// query input and render the TUI.
pub struct Terminal {
    /// The width and height of the terminal we're rendering in.
    pub size: (u16, u16),
    /// The current location of the mouse.
    pub mouse_pos: (u16, u16),
    /// Mouse buttons that have just been clicked.
    ///
    /// Mouse buttons are stored as a u8, but only buttons 0-11 are actually
    /// supported (other buttons aren't always communicated by the terminal).
    /// Mouse buttons are indexed starting at 0 (IE, mouse button 0 is left
    /// click).
    pub clicked_mouse_buttons: HashSet<u8>,
    /// Mouse buttons that are currently being held.
    ///
    /// Mouse buttons are stored as a u8, but only buttons 0-11 are actually
    /// supported (other buttons aren't always communicated by the terminal).
    /// Mouse buttons are indexed starting at 0 (IE, mouse button 0 is left
    /// click).
    pub held_mouse_buttons: HashSet<u8>,
    /// Mouse buttons that have just been released.
    ///
    /// Mouse buttons are stored as a u8, but only buttons 0-11 are actually
    /// supported (other buttons aren't always communicated by the terminal).
    /// Mouse buttons are indexed starting at 0 (IE, mouse button 0 is left
    /// click).
    pub released_mouse_buttons: HashSet<u8>,
    /// Scroll direction.
    pub scroll_direction: Option<ScrollDirection>,
    /// Any actively held modifier keys.
    pub modifier_keys: ModifierKeys,
    /// Keys currently held by the user.
    pub pressed_keys: HashSet<Key>,
    /// If we should exit the app.
    pub exit: bool,
    /// The location to move the cursor to, if one was set.
    pub target_cursor_location: Cell<Option<(u16, u16)>>,
    /// The buffer for reading from stdin.
    input_buffer: Vec<u8>,
    /// The buffer for writing to stdout.
    output_buffer: ArenaVec<u8>,
}
impl Terminal {
    pub fn set_fg(&self, fg: Option<Colour>) {
        let mut buffer = &self.output_buffer;

        if let Some(fg) = fg {
            // Custom RGB colour
            // TODO: Support older colour formats for terminals that don't
            // support RGB
            write!(buffer, "\x1B[38;2;{};{};{}m", fg.r, fg.g, fg.b).unwrap();
        } else {
            // Default fg colour
            buffer.extend_from_slice(b"\x1B[39m");
        }
    }
    pub fn set_bg(&self, bg: Option<Colour>) {
        let mut buffer = &self.output_buffer;

        if let Some(bg) = bg {
            // Custom RGB colour
            // TODO: Support older colour formats for terminals that don't
            // support RGB
            write!(buffer, "\x1B[48;2;{};{};{}m", bg.r, bg.g, bg.b).unwrap();
        } else {
            // Default bg colour
            buffer.extend_from_slice(b"\x1B[49m");
        }
    }

    #[inline(always)]
    pub fn draw<E: Shape>(&self, element: E) -> E::Output {
        element.draw(self)
    }

    pub fn render_bytes(&self, bytes: &[u8], position: (u16, u16)) {
        let mut buffer = &self.output_buffer;

        // Move cursor
        write!(buffer, "\x1B[{};{}H", position.1 + 1, position.0 + 1).unwrap();
        // Print bytes
        buffer.extend_from_slice(bytes);
    }
    pub fn render_char(&self, figure: char, position: (u16, u16)) {
        let mut buf = [0; 4];
        let string = figure.encode_utf8(&mut buf);
        self.render_bytes(string.as_bytes(), position)
    }
    pub fn render_string(&self, string: &str, position: (u16, u16)) {
        self.render_bytes(string.as_bytes(), position)
    }
    pub fn render_string_unpositioned(&self, string: &str) {
        self.output_buffer.extend_from_slice(string.as_bytes());
    }

    pub fn update(&mut self) {
        print!("\x1B[0m\x1B[2J\x1B[H");
        stdout().flush().unwrap();
        if let Some((x, y)) = self.target_cursor_location.take() {
            // Move cursor
            write!(&self.output_buffer, "\x1B[{};{}H", y + 1, x + 1).unwrap();
            // Show cursor
            write!(&self.output_buffer, "\x1B[?25h").unwrap();
        } else {
            // Hide cursor
            write!(&self.output_buffer, "\x1B[?25l").unwrap();
        }
        stdout().write_all(&self.output_buffer).unwrap();
        stdout().flush().unwrap();
        self.output_buffer.clear();

        os::set_blocking(false);

        // Get terminal size
        self.size = os::get_terminal_size();

        // Clear old user input
        self.pressed_keys.clear();

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

        // Progress mouse button states
        for btn in self.clicked_mouse_buttons.drain() {
            self.held_mouse_buttons.insert(btn);
        }
        self.released_mouse_buttons.clear();

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
                                self.modifier_keys.shift = (btn & 0b0000_0100) != 0;
                                self.modifier_keys.meta = (btn & 0b0000_1000) != 0;
                                self.modifier_keys.control = (btn & 0b0001_0000) != 0;

                                if button_number == 4 {
                                    self.scroll_direction = Some(ScrollDirection::Backwards);
                                } else if button_number == 5 {
                                    self.scroll_direction = Some(ScrollDirection::Forwards);
                                } else {
                                    // -1 cause it starts indexing pixels at 1
                                    self.mouse_pos = (x - 1, y - 1);
                                    let btn = button_number as u8;
                                    if clicked {
                                        if !self.held_mouse_buttons.contains(&btn) {
                                            self.clicked_mouse_buttons.insert(btn);
                                        }
                                    } else {
                                        self.clicked_mouse_buttons.remove(&btn);
                                        self.held_mouse_buttons.remove(&btn);
                                        self.released_mouse_buttons.insert(btn);
                                    }
                                }
                            }

                            // Arrow keys
                            b'A' => {
                                self.pressed_keys.insert(Key::ArrowUp);
                            }
                            b'B' => {
                                self.pressed_keys.insert(Key::ArrowDown);
                            }
                            b'C' => {
                                self.pressed_keys.insert(Key::ArrowRight);
                            }
                            b'D' => {
                                self.pressed_keys.insert(Key::ArrowLeft);
                            }

                            // Group of special keys that end with ~
                            other if stdin.next().map(|(_, byte)| byte) == Some(b'~') => {
                                match other {
                                    b'5' => {
                                        self.pressed_keys.insert(Key::PageUp);
                                    }
                                    b'6' => {
                                        self.pressed_keys.insert(Key::PageDown);
                                    }
                                    b'1' | b'7' => {
                                        self.pressed_keys.insert(Key::Home);
                                    }
                                    b'4' | b'8' => {
                                        self.pressed_keys.insert(Key::End);
                                    }
                                    b'3' => {
                                        self.pressed_keys.insert(Key::Delete);
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
                                self.pressed_keys.insert(Key::Home);
                            }
                            b'F' => {
                                self.pressed_keys.insert(Key::End);
                            }
                            b'O' => {
                                let Some((_, next)) = stdin.next() else {
                                    println!("WARN: Got incomplete control key sequence ESC[O");
                                    continue;
                                };
                                match next {
                                    b'H' => {
                                        self.pressed_keys.insert(Key::Home);
                                    }
                                    b'F' => {
                                        self.pressed_keys.insert(Key::End);
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
                    for char in text.chars() {
                        if char == '\x7F' {
                            self.pressed_keys.insert(Key::Backspace);
                        } else {
                            self.pressed_keys.insert(Key::Text(char));
                        }
                    }
                }
            }
        }

        os::set_blocking(true);
    }

    /// Called when the [`Terminal`] is dropped, or when the program panics, to
    /// reset the terminal & undo all the things Scaffolding changed.
    pub fn on_drop() {
        // Running this code twice can cause weird terminal issues
        if TERMINAL_DROPPED.swap(true, Ordering::Release) {
            return;
        }

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

        // Set a panic handler to leave the alternate buffer before printing
        // the panic message
        // Otherwise the message will be printed inside the alternate buffer,
        // and then we leave the alternate buffer when Terminal is dropped,
        // so the message can't be seen.
        let normal_panic_handler = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |panic_info| {
            Terminal::on_drop();
            normal_panic_handler(panic_info);
        }));

        Self {
            size: (0, 0),
            mouse_pos: (0, 0),
            modifier_keys: ModifierKeys::default(),
            scroll_direction: None,
            clicked_mouse_buttons: HashSet::default(),
            held_mouse_buttons: HashSet::default(),
            released_mouse_buttons: HashSet::default(),
            pressed_keys: HashSet::default(),
            exit: false,
            target_cursor_location: Cell::new(None),
            input_buffer: Vec::with_capacity(10),
            output_buffer: ArenaVec::with_reserved_memory(MemoryAmount::Megabytes(1).into_bytes()),
        }
    }
}
impl Drop for Terminal {
    fn drop(&mut self) {
        Self::on_drop();
    }
}
