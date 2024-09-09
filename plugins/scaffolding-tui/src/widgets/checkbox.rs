use {
    super::{Frame, HAlign, HorizontalOverflowStyle, Text, Widget},
    crate::{prelude::Terminal, Colour},
    scaffolding::{
        datatypes::uniq::UniqKey,
        world::{Executable, ExecutableWithState, Singleton, TypeErasedExecutable, Uniqs},
    },
};

#[derive(Default)]
struct CheckboxCache {
    checked: bool,
    mouse_held: bool,
}

pub struct Checkbox<'a> {
    label: &'a str,
    cache_key: UniqKey,
    frame: Frame,
    checked_char: char,
    unchecked_char: char,
    text_colour: Option<Colour>,
}
impl<'a> Checkbox<'a> {
    pub fn new(label: &'a str, cache_key: UniqKey) -> Self {
        Self {
            label,
            cache_key,
            frame: Frame {
                x: 0,
                y: 0,
                width: 10,
                height: 1,
            },
            checked_char: '🗹',
            unchecked_char: '☐',
            text_colour: None,
        }
    }

    pub fn label(mut self, label: &'a str) -> Self {
        self.label = label;
        self
    }

    pub fn checked_char(mut self, char: char) -> Self {
        self.checked_char = char;
        self
    }
    pub fn unchecked_char(mut self, char: char) -> Self {
        self.unchecked_char = char;
        self
    }

    pub fn text_colour(mut self, colour: Option<Colour>) -> Self {
        self.text_colour = colour;
        self
    }

    fn draw(self, uniqs: &Uniqs, terminal: &Singleton<Terminal>) -> CheckboxOut {
        let clicked = self.clicked(terminal);
        let cache: &mut CheckboxCache = uniqs.get(self.cache_key);

        if clicked {
            if !cache.mouse_held {
                cache.checked = !cache.checked;
                cache.mouse_held = true;
            }
        } else {
            cache.mouse_held = false;
        }

        if self.frame.width > 0 {
            let char = if cache.checked {
                self.checked_char
            } else {
                self.unchecked_char
            };
            terminal.set_fg(self.text_colour);
            terminal.render_char(char, (self.frame.x, self.frame.y));

            if self.frame.width > 2 {
                terminal.draw(
                    Text::new(self.label)
                        .x(self.frame.x + 2)
                        .y(self.frame.y)
                        .width(self.frame.width.saturating_sub(2))
                        .height(1)
                        .horizontal_anchor(HAlign::Left)
                        .horizontal_overflow(HorizontalOverflowStyle::ClipWithChar('…')),
                );
            }
        }

        CheckboxOut {
            checked: cache.checked,
        }
    }
}
impl_frame_methods!(Checkbox<'_>, x, y, width, clicked, hovered);

pub struct CheckboxOut {
    pub checked: bool,
}

impl<'a> Widget<'a> for Checkbox<'a> {
    type Output = CheckboxOut;

    fn build_draw_fn(self) -> impl TypeErasedExecutable<'a, Output = Self::Output> {
        Self::draw.with_state(self).type_erase()
    }
}
