use {
    super::{HAlign, HorizontalOverflowStyle, Text, TextStyleFlags, VAlign},
    crate::{
        input::Key,
        prelude::Terminal,
        shapes::*,
        widgets::{Frame, Widget},
        Colour,
    },
    scaffolding::{
        datatypes::uniq::UniqKey,
        world::{Executable, ExecutableWithState, Singleton, TypeErasedExecutable, Uniqs},
    },
    unicode_segmentation::UnicodeSegmentation,
};

#[derive(Default)]
struct TextInputCache {
    /// The position of the cursor in this text input. This is in graphemes, not
    /// bytes.
    cursor_pos: usize,
    /// If the text input is currently focused
    focused: bool,
    /// An offset into the string to start rendering at. This is used, for
    /// example, when the string is longer than the text input's length and the
    /// user has scrolled over to a part of the string that's past the text
    /// input's length.
    render_offset: usize,
}

pub struct TextInputOut {
    pub focused: bool,
}

pub struct TextInput<'a> {
    buffer: &'a mut String,
    placeholder: Option<&'a str>,
    placeholder_colour: Option<Colour>,
    frame: Frame,
    cache_key: Option<UniqKey>,
    border_style: Option<BorderStyle>,
    border_colour: Option<Colour>,
    text_colour: Option<Colour>,
    background_colour: Option<Colour>,
    text_style: TextStyleFlags,
}
impl<'a> TextInput<'a> {
    pub fn new(buffer: &'a mut String, cache_key: UniqKey) -> Self {
        Self {
            buffer,
            placeholder: None,
            placeholder_colour: Some(Colour::GREY),
            frame: Frame {
                x: 0,
                y: 0,
                width: 10,
                height: 3,
            },
            cache_key: Some(cache_key),
            border_style: Some(BorderStyle::ROUND),
            border_colour: None,
            text_colour: None,
            background_colour: None,
            text_style: TextStyleFlags::default(),
        }
    }

    pub fn border(mut self, style: Option<BorderStyle>) -> Self {
        self.border_style = style;
        self
    }
    pub fn placeholder(mut self, placeholder: &'a str) -> Self {
        self.placeholder = Some(placeholder);
        self
    }
    pub fn text_style(mut self, style: impl Into<TextStyleFlags>) -> Self {
        self.text_style.merge(style.into());
        self
    }

    fn draw(mut self, uniqs: &Uniqs, terminal: &Singleton<Terminal>) -> TextInputOut {
        let cache: &mut TextInputCache = uniqs.get(self.cache_key.take().unwrap());

        let text_offset = if self.border_style.is_some() { 1 } else { 0 };
        self.frame.height = if self.border_style.is_some() { 3 } else { 1 };

        // On a mouse press, if the press was inside the text input, focus it
        // Otherwise, unfocus it
        if terminal.clicked_mouse_buttons.contains(&0) {
            cache.focused = self.frame.contains(terminal.mouse_pos);
        }

        if cache.focused {
            for key in terminal.pressed_keys.iter() {
                self.handle_keypress(cache, *key);
            }

            let target_cursor_x = self.frame.x + (cache.cursor_pos - cache.render_offset) as u16;
            terminal.target_cursor_location.set(Some((
                target_cursor_x + text_offset,
                self.frame.y + text_offset,
            )));
        }

        terminal.set_bg(self.background_colour);

        let string = if !self.buffer.is_empty() {
            terminal.set_fg(self.text_colour);
            self.buffer as &'a str
        } else {
            terminal.set_fg(self.placeholder_colour);
            self.placeholder.unwrap_or_default()
        };

        let mut string_graphemes = string.grapheme_indices(true);
        let string_render_start_idx = string_graphemes
            .nth(cache.render_offset)
            .map(|(idx, _val)| idx)
            .unwrap_or(0);
        let string_render_end_idx = string_graphemes
            .nth(self.max_renderable_graphemes() as usize)
            .map(|(idx, _val)| idx)
            .unwrap_or(string.len());

        terminal.draw(
            Text::new(&string[string_render_start_idx..string_render_end_idx])
                .frame(self.frame)
                .x(self.frame.x + text_offset)
                .y(self.frame.y + text_offset)
                .horizontal_overflow(HorizontalOverflowStyle::Clip)
                .vertical_anchor(VAlign::Center)
                .horizontal_anchor(HAlign::Left)
                .text_style(self.text_style),
        );

        if let Some(style) = self.border_style {
            terminal.set_fg(self.border_colour);
            terminal.draw(Border {
                x: self.frame.x,
                y: self.frame.y,
                width: self.frame.width,
                height: self.frame.height,
                style,
            });
        }

        TextInputOut {
            focused: cache.focused,
        }
    }

    fn handle_keypress(&mut self, cache: &mut TextInputCache, key: Key) {
        match key {
            Key::Text(char) => {
                // Insert the character at the correct byte in our buffer,
                // based on the cursor's location
                let cursor_byte_idx = self
                    .buffer
                    .grapheme_indices(true)
                    .nth(cache.cursor_pos)
                    .map(|(idx, _)| idx)
                    .unwrap_or(self.buffer.len());
                self.buffer.insert(cursor_byte_idx, char);

                // Check if we're in the middle of the string and need to move
                // the cursor, or if we've filled the text input and need to
                // scroll
                if (cache.cursor_pos - cache.render_offset)
                    == self.max_renderable_graphemes() as usize
                {
                    cache.render_offset += 1;
                    cache.cursor_pos += 1;
                } else if cache.cursor_pos < self.buffer.graphemes(true).count() {
                    cache.cursor_pos += 1;
                }
            }
            Key::ArrowLeft => {
                // Check if we need to scroll
                if cache.cursor_pos == cache.render_offset {
                    cache.render_offset = cache.render_offset.saturating_sub(1)
                }

                // Move the cursor back one
                cache.cursor_pos = cache.cursor_pos.saturating_sub(1);
            }
            Key::ArrowRight => {
                // Don't go past the end of the text
                if cache.cursor_pos >= self.buffer.graphemes(true).count() {
                    return;
                }

                // Check if we need to scroll
                if cache.cursor_pos
                    == cache.render_offset + self.max_renderable_graphemes() as usize
                {
                    cache.render_offset += 1;
                }
                // Move the cursor forwards one
                cache.cursor_pos = cache.cursor_pos.saturating_add(1);
            }
            Key::Backspace => {
                if let Some(grapheme) = cache.cursor_pos.checked_sub(1) {
                    let idx = self.buffer.grapheme_indices(true).nth(grapheme).unwrap().0;
                    self.buffer.remove(idx);
                    cache.cursor_pos -= 1;
                    cache.render_offset = cache.render_offset.saturating_sub(1);
                }
            }
            Key::Delete => {
                if cache.cursor_pos < self.buffer.graphemes(true).count() {
                    let idx = self
                        .buffer
                        .grapheme_indices(true)
                        .nth(cache.cursor_pos)
                        .unwrap()
                        .0;
                    self.buffer.remove(idx);
                    cache.render_offset = cache.render_offset.saturating_sub(1);
                }
            }
            _ => {}
        }
    }
    fn max_renderable_graphemes(&self) -> u16 {
        if self.border_style.is_some() {
            self.frame.width.saturating_sub(2)
        } else {
            self.frame.width
        }
    }
}
impl<'a> Widget<'a> for TextInput<'a> {
    type Output = TextInputOut;

    fn build_draw_fn(self) -> impl TypeErasedExecutable<'a, Output = Self::Output> {
        Self::draw.with_state(self).type_erase()
    }
}
impl_frame_methods!(TextInput<'_>, x, y, width, hovered, clicked);
impl_colour_methods!(TextInput<'_>, text_colour, border_colour, background_colour);
