use std::fmt::{Display, Write};

#[derive(Hash, PartialEq, Eq, Clone, Copy, Debug)]
pub enum Key {
    // a, b, c, etc; or a non-ascii character, such as か
    Text(char),

    // arrows
    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,

    // misc
    Escape,
    Delete,
    Backspace,
    PageUp,
    PageDown,
    Home,
    End,
}
impl Display for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Text(char) => f.write_char(*char),

            Self::ArrowUp => f.write_str("up"),
            Self::ArrowDown => f.write_str("down"),
            Self::ArrowLeft => f.write_str("left"),
            Self::ArrowRight => f.write_str("right"),

            Self::Escape => f.write_str("esc"),
            Self::Delete => f.write_str("delete"),
            Self::Backspace => f.write_str("backspace"),
            Self::PageUp => f.write_str("page-up"),
            Self::PageDown => f.write_str("page-down"),
            Self::Home => f.write_str("home"),
            Self::End => f.write_str("end"),
        }
    }
}

#[derive(Default, Hash, PartialEq, Eq, Clone, Copy, Debug)]
pub struct ModifierKeys {
    pub shift: bool,
    pub meta: bool,
    pub control: bool,
}
#[derive(Hash, PartialEq, Eq, Clone, Copy, Debug)]
pub enum ScrollDirection {
    /// Scroll so text that was previously off the top of the screen is now
    /// visible.
    Backwards,
    /// Scroll so text that was previously off the bottom of the screen is now
    /// visible.
    Forwards,
}
