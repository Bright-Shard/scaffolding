use {
    super::*,
    crate::{prelude::Terminal, shapes::*, Colour},
    scaffolding::datatypes::uniq::UniqKey,
};

#[derive(Debug)]
pub struct ButtonOut {
    /// The current state of the button.
    pub state: ButtonState,
    /// The mouse cursor is over the button.
    pub hovered: bool,
}
#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum ButtonState {
    /// The mouse has just clicked on the button.
    Pressed,
    /// The mouse has been clicking the button for multiple frames.
    Held,
    /// The mouse just released the button.
    Released,
    /// The mouse isn't interacting with the button.
    Inactive,
}
impl Default for ButtonState {
    fn default() -> Self {
        Self::Inactive
    }
}

pub struct Button<'a> {
    label: &'a str,
    border_style: Option<BorderStyle>,
    background_colour: Option<Colour>,
    border_colour: Option<Colour>,
    text_colour: Option<Colour>,
    frame: Frame,
    key: UniqKey,
}
impl<'a> Button<'a> {
    pub fn new(label: &'a str, key: UniqKey) -> Self {
        Self {
            label,
            border_style: Some(BorderStyle::ROUND),
            border_colour: None,
            text_colour: None,
            background_colour: None,
            frame: Frame {
                x: 0,
                y: 0,
                width: 8,
                height: 3,
            },
            key,
        }
    }

    pub fn label(mut self, label: &'a str) -> Self {
        self.label = label;
        self
    }
    pub fn border(mut self, style: Option<BorderStyle>) -> Self {
        self.border_style = style;
        self
    }

    fn draw(mut self, terminal: &Singleton<Terminal>, uniqs: &Uniqs) -> ButtonOut {
        terminal.set_bg(self.background_colour);

        if let Some(style) = self.border_style.take() {
            terminal.set_fg(self.border_colour);
            terminal.draw(Border {
                x: self.frame.x,
                y: self.frame.y,
                width: self.frame.width,
                height: self.frame.height,
                style,
            });
        }

        terminal.draw(
            Text::new(self.label)
                // TODO: Implement wrapping text
                .horizontal_overflow(HorizontalOverflowStyle::Clip)
                .frame(self.frame)
                .text_colour(self.text_colour)
                .background_colour(self.background_colour),
        );

        let hovered = self.hovered(terminal);
        let cached_state: &mut ButtonState = uniqs.get(self.key);

        if hovered {
            if terminal.pressed_mouse_buttons.contains(&0) {
                match *cached_state {
                    ButtonState::Pressed => {
                        *cached_state = ButtonState::Held;
                    }
                    ButtonState::Inactive | ButtonState::Released => {
                        *cached_state = ButtonState::Pressed;
                    }
                    ButtonState::Held => {}
                }
            } else {
                match *cached_state {
                    ButtonState::Pressed | ButtonState::Held => {
                        *cached_state = ButtonState::Released;
                    }
                    ButtonState::Released => {
                        *cached_state = ButtonState::Inactive;
                    }
                    ButtonState::Inactive => {}
                }
            }
        }

        let state = *cached_state;

        ButtonOut { state, hovered }
    }
}
impl<'a> Widget<'a> for Button<'a> {
    type Output = ButtonOut;

    fn build_draw_fn(self) -> impl TypeErasedExecutable<'a, Output = Self::Output> {
        Self::draw.with_state(self).type_erase()
    }
}
impl_frame_methods!(Button<'_>);
impl_colour_methods!(Button<'_>, text_colour, border_colour, background_colour);
