use scaffolding::plugin_prelude::*;

/// A type that can be rendered in the terminal.
pub trait Widget<'a> {
    /// The type that this widget's draw function will return.
    type Output: 'a;

    /// Build the draw function, which is an executable.
    fn build_draw_fn(self) -> impl TypeErasedExecutable<'a, Output = Self::Output>;
}
impl<'a, E: TypeErasedExecutable<'a>> Widget<'a> for E {
    type Output = E::Output;

    fn build_draw_fn(self) -> impl TypeErasedExecutable<'a, Output = Self::Output> {
        self
    }
}

/// A rectangular area in the terminal. This is generally used for widget
/// positioning.
pub struct Frame {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}
impl Frame {
    /// Check if this [`Frame`] contains the given `(x, y)` point.
    pub fn contains(&self, pos: (u16, u16)) -> bool {
        let (x, y) = pos;
        x >= self.x && y >= self.y && x < (self.x + self.width) && y < (self.y + self.height)
    }
}

/// Vertical alignment values.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum VAlign {
    Top,
    Center,
    Bottom,
}
impl Default for VAlign {
    fn default() -> Self {
        Self::Center
    }
}
/// Horizontal alignment values.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum HAlign {
    Left,
    Center,
    Right,
}
impl Default for HAlign {
    fn default() -> Self {
        Self::Center
    }
}

/// Adds common methods to widgets with a frame, such as:
/// .x
/// .y
/// .width
/// .height
/// .frame
/// .hovered
/// .clicked
///
/// The frame must be stored in `self.frame`.
///
/// If you only want some of these methods, just provide a list, eg:
/// impl_frame_methods!(TextInput<'_>, x, y, width, hovered, clicked)
/// Will add all of those methods to `TextInput` except .height
macro_rules! impl_frame_methods {
    ($ty:tt) => {
        impl_frame_methods!($ty, x, y, width, height, frame, hovered, clicked);
    };
    ($ty:tt, $($method:ident),*) => {
        $(impl_frame_methods!(@$method $ty);)*
    };
    ($ty:tt<'_>) => {
        impl_frame_methods!(($ty<'_>), x, y, width, height, frame, hovered, clicked);
    };
    ($ty:tt<'_>, $($method:ident),*) => {
        $(impl_frame_methods!(@$method ($ty<'_>));)*
    };
    (@x $ty:tt) => {
        #[allow(unused_parens)]
        impl $ty {
            pub fn x(mut self, x: u16) -> Self {
                self.frame.x = x;
                self
            }
        }
    };
    (@y $ty:tt) => {
        #[allow(unused_parens)]
        impl $ty {
            pub fn y(mut self, y: u16) -> Self {
                self.frame.y = y;
                self
            }
        }
    };
    (@width $ty:tt) => {
        #[allow(unused_parens)]
        impl $ty {
            pub fn width(mut self, width: u16) -> Self {
                self.frame.width = width;
                self
            }
        }
    };
    (@height $ty:tt) => {
        #[allow(unused_parens)]
        impl $ty {
            pub fn height(mut self, height: u16) -> Self {
                self.frame.height = height;
                self
            }
        }
    };
    (@frame $ty:tt) => {
        #[allow(unused_parens)]
        impl $ty {
            pub fn frame(mut self, frame: Frame) -> Self {
                self.frame = frame;
                self
            }
        }
    };
    (@hovered $ty:tt) => {
        #[allow(unused_parens)]
        impl $ty {
            pub fn hovered(&self, terminal: &Terminal) -> bool {
                self.frame.contains(terminal.mouse_pos)
            }
        }
    };
    (@clicked $ty:tt) => {
        #[allow(unused_parens)]
        impl $ty {
            pub fn clicked(&self, terminal: &Terminal) -> bool {
                terminal.pressed_mouse_buttons.contains(&0) && self.frame.contains(terminal.mouse_pos)
            }
        }
    };
}

/// Adds colour methods to widgets. For each field listed, it'll generate a
/// method with the same name. For example:
///
/// impl_colour_methods!(Widget, background_colour, text_colour);
///
/// generates
///
/// impl Widget {
///     pub fn background_colour(mut self, colour: Option<Colour>) -> Self {
///         self.background_colour = colour;
///         self
///     }
///     pub fn text_colour(mut self, colour: Option<Colour>) -> Self {
///         self.text_colour = colour;
///         self
///     }
/// }
///
/// Thus the field name will have to match the method name.
macro_rules! impl_colour_methods {
    ($widget:ident, $($field:ident),*) => {
        impl $widget {
            $(
            pub fn $field(mut self, colour: Option<Colour>) -> Self {
                self.$field = colour;
                self
            }
            )*
        }
    };
    ($widget:ident<'_>, $($field:ident),*) => {
        impl $widget<'_> {
            $(
            pub fn $field(mut self, colour: Option<Colour>) -> Self {
                self.$field = colour;
                self
            }
            )*
        }
    };
}

// Default/included widgets
// They're down here because macros don't work like normal functions... and
// can't be used until after they're defined
// So if they were `mod`ed at the top of the file these submodules couldn't
// use the macros in this file

mod button;
pub use button::*;
mod textinput;
pub use textinput::*;
mod text;
pub use text::*;
