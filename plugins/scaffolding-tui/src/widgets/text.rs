use {
    super::{HAlign, VAlign, Widget},
    crate::{
        shapes::{RawString, Shape},
        terminal::Terminal,
        widgets::Frame,
        Colour,
    },
    scaffolding::world::{Executable, ExecutableWithState, Singleton, TypeErasedExecutable},
};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum HorizontalOverflowStyle {
    /// Cut off any text that goes past the widget.
    Clip,
    /// Cut off any text that goes past the widget, placing the specified char
    /// at the end of the text to show it's been clipped.
    ClipWithChar(char),
    /// Wrap text that goes past the widget onto a new line.
    Wrap,
    /// Allow text to render past the edge of the widget.
    Overflow,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum VerticalOverflowStyle {
    /// Cut off any text that goes past the widget.
    Clip,
    /// Cut off any text that goes past the widget, placing the specified char
    /// at the end of the text to show it's been clipped.
    ClipWithChar(char),
    /// Allow text to render past the edge of the widget.
    Overflow,
}

pub struct Text<'a> {
    text: &'a str,
    frame: Frame,
    text_colour: Option<Colour>,
    background_colour: Option<Colour>,
    vertical_anchor: VAlign,
    vertical_overflow: VerticalOverflowStyle,
    horizontal_anchor: HAlign,
    horizontal_overflow: HorizontalOverflowStyle,
}
impl<'a> Text<'a> {
    pub fn new(text: &'a str) -> Self {
        Self {
            text,
            frame: Frame {
                x: 0,
                y: 0,
                width: 10,
                height: 10,
            },
            text_colour: None,
            background_colour: None,
            vertical_anchor: VAlign::Center,
            vertical_overflow: VerticalOverflowStyle::Clip,
            horizontal_anchor: HAlign::Center,
            horizontal_overflow: HorizontalOverflowStyle::Wrap,
        }
    }

    pub fn vertical_anchor(mut self, align: VAlign) -> Self {
        self.vertical_anchor = align;
        self
    }
    pub fn vertical_overflow(mut self, overflow: VerticalOverflowStyle) -> Self {
        self.vertical_overflow = overflow;
        self
    }
    pub fn horizontal_anchor(mut self, align: HAlign) -> Self {
        self.horizontal_anchor = align;
        self
    }
    pub fn horizontal_overflow(mut self, overflow: HorizontalOverflowStyle) -> Self {
        self.horizontal_overflow = overflow;
        self
    }

    fn draw(self, terminal: &Singleton<Terminal>) {
        match self.horizontal_overflow {
            HorizontalOverflowStyle::Overflow => {
                let horizontal_diff = self.frame.width.saturating_sub(self.text.len() as u16);
                let x = if horizontal_diff > 0 {
                    match self.horizontal_anchor {
                        HAlign::Left => self.frame.x,
                        HAlign::Center => self.frame.x + (horizontal_diff / 2),
                        HAlign::Right => self.frame.x + horizontal_diff,
                    }
                } else {
                    self.frame.x
                };

                let y = match self.vertical_anchor {
                    VAlign::Top => self.frame.y,
                    VAlign::Center => self.frame.y + (self.frame.height / 2),
                    VAlign::Bottom => self.frame.y + (self.frame.height - 1),
                };

                terminal.draw(RawString {
                    x,
                    y,
                    text: self.text,
                })
            }
            HorizontalOverflowStyle::Clip => {
                let horizontal_diff = self.frame.width.saturating_sub(self.text.len() as u16);
                let x = if horizontal_diff > 0 {
                    match self.horizontal_anchor {
                        HAlign::Left => self.frame.x,
                        HAlign::Center => self.frame.x + (horizontal_diff / 2),
                        HAlign::Right => self.frame.x + horizontal_diff,
                    }
                } else {
                    self.frame.x
                };

                let y = match self.vertical_anchor {
                    VAlign::Top => self.frame.y,
                    VAlign::Center => self.frame.y + (self.frame.height / 2),
                    VAlign::Bottom => self.frame.y + (self.frame.height - 1),
                };

                if self.text.len() > self.frame.width as usize {
                    terminal.draw(RawString {
                        x,
                        y,
                        text: &self.text[0..self.frame.width as usize],
                    });
                } else {
                    terminal.draw(RawString {
                        x,
                        y,
                        text: &self.text,
                    });
                }
            }
            HorizontalOverflowStyle::ClipWithChar(char) => {
                let horizontal_diff = self.frame.width.saturating_sub(self.text.len() as u16);
                let x = if horizontal_diff > 0 {
                    match self.horizontal_anchor {
                        HAlign::Left => self.frame.x,
                        HAlign::Center => self.frame.x + (horizontal_diff / 2),
                        HAlign::Right => self.frame.x + horizontal_diff,
                    }
                } else {
                    self.frame.x
                };

                let y = match self.vertical_anchor {
                    VAlign::Top => self.frame.y,
                    VAlign::Center => self.frame.y + (self.frame.height / 2),
                    VAlign::Bottom => self.frame.y + (self.frame.height - 1),
                };

                if self.text.len() > self.frame.width as usize {
                    terminal.draw(RawString {
                        x,
                        y,
                        text: &self.text[0..self.frame.width.saturating_sub(1) as usize],
                    });
                    terminal.render_char(
                        char,
                        (
                            (self.frame.x + self.frame.width).saturating_sub(2),
                            self.frame.y,
                        ),
                    );
                } else {
                    terminal.draw(RawString {
                        x,
                        y,
                        text: &self.text,
                    });
                }
            }
            HorizontalOverflowStyle::Wrap => {
                todo!()
            }
        }
    }
}
impl<'a> Widget<'a> for Text<'a> {
    type Output = ();

    fn build_draw_fn(self) -> impl TypeErasedExecutable<'a, Output = Self::Output> {
        Self::draw.with_state(self).type_erase()
    }
}
impl Shape for Text<'_> {
    type Output = ();

    fn draw(self, terminal: &Terminal) -> Self::Output {
        self.draw(&Singleton::new(terminal))
    }
}
impl_frame_methods!(Text<'_>);
impl_colour_methods!(Text<'_>, text_colour, background_colour);
