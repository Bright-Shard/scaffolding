use crate::prelude::Terminal;

pub trait Shape {
    type Output;

    fn draw(self, terminal: &Terminal) -> Self::Output;
}

// Note: There's also the `Text` shape, defined in `widgets/text.rs`
// It's both a Shape and a Widget

pub struct RawString<S: AsRef<str>> {
    pub x: u16,
    pub y: u16,
    pub text: S,
}
impl<S: AsRef<str>> Shape for RawString<S> {
    type Output = ();

    fn draw(self, terminal: &Terminal) -> Self::Output {
        terminal.render_string(self.text.as_ref(), (self.x, self.y));
    }
}

/// A single-colour rectangle.
pub struct Rect {
    /// The x-coordinate for the top left of the rectangle.
    pub x: u16,
    /// The y-coordinate for the top left of the rectangle.
    pub y: u16,
    /// How wide the rectangle is.
    pub width: u16,
    /// How tall the rectangle is.
    pub height: u16,
}
impl Shape for Rect {
    type Output = ();

    fn draw(self, terminal: &Terminal) -> Self::Output {
        let row = " ".repeat(self.width as usize);
        for current_row in 0..self.height {
            terminal.render_string(&row, (self.x, self.y + current_row))
        }
    }
}

/// A border that can go around another UI element. The characters that are
/// used in the border are determined by the [`BorderStyle`] used.
pub struct Border {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
    pub style: BorderStyle,
}
impl Shape for Border {
    type Output = ();

    fn draw(self, terminal: &Terminal) -> Self::Output {
        let btm: String = (0..self.width - 2).map(|_| self.style.bottom).collect();
        let top: String = (0..self.width - 2).map(|_| self.style.top).collect();

        // top & top corners
        terminal.render_char(self.style.top_left, (self.x, self.y));
        terminal.render_string(&top, (self.x + 1, self.y));
        terminal.render_char(self.style.top_right, (self.x + self.width - 1, self.y));

        // bottom & bottom corners
        terminal.render_char(
            self.style.bottom_right,
            (self.x + self.width - 1, self.y + self.height - 1),
        );
        terminal.render_string(&btm, (self.x + 1, self.y + self.height - 1));
        terminal.render_char(self.style.bottom_left, (self.x, self.y + self.height - 1));

        // sides
        for height in 1..self.height - 1 {
            terminal.render_char(self.style.left, (self.x, self.y + height));
            terminal.render_char(self.style.right, (self.x + self.width - 1, self.y + height));
        }
    }
}

/// The characters used to make a [`Border`]. There are several included
/// styles in this type's associated constants; it may be easier to use those
/// than to make your own.
#[derive(Clone)]
pub struct BorderStyle {
    pub top_left: char,
    pub top: char,
    pub top_right: char,
    pub right: char,
    pub bottom_right: char,
    pub bottom: char,
    pub bottom_left: char,
    pub left: char,
}
impl BorderStyle {
    /// *---*
    /// |   |
    /// *---*
    pub const ASCII: Self = Self {
        top_left: '*',
        top: '-',
        top_right: '*',
        right: '|',
        bottom_right: '*',
        bottom: '-',
        bottom_left: '*',
        left: '|',
    };
    /// ╭───╮
    /// │   │
    /// ╰───╯
    pub const ROUND: Self = Self {
        top_left: '╭',
        top: '─',
        top_right: '╮',
        right: '│',
        bottom_right: '╯',
        bottom: '─',
        bottom_left: '╰',
        left: '│',
    };
    /// ┌───┐
    /// │   │
    /// └───┘
    pub const NORMAL: Self = Self {
        top_left: '┌',
        top: '─',
        top_right: '┐',
        right: '│',
        bottom_right: '┘',
        bottom: '─',
        bottom_left: '└',
        left: '│',
    };
    /// ┏━━━┓
    /// ┃   ┃
    /// ┗━━━┛
    pub const HEAVY: Self = Self {
        top_left: '┏',
        top: '━',
        top_right: '┓',
        right: '┃',
        bottom_right: '┛',
        bottom: '━',
        bottom_left: '┗',
        left: '┃',
    };
    /// ╔═══╗
    /// ║   ║
    /// ╚═══╝
    pub const DOUBLE: Self = Self {
        top_left: '╔',
        top: '═',
        top_right: '╗',
        right: '║',
        bottom_right: '╝',
        bottom: '═',
        bottom_left: '╚',
        left: '║',
    };
}
