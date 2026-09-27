use alloc::{string::String, vec};
use embedded_graphics::{prelude::*, primitives::Rectangle};

use crate::display::{
    style::{Align, Flexbox, Insets, Style},
    text_style::TextStyle,
    theme::{FromTheme, Theme, ThemeColor},
};

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScrollState {
    offset: usize,
}

pub struct ListStyle<Color: PixelColor> {
    pub normal: Style<Color>,
    pub active: Style<Color>,
}

impl<Color: PixelColor + FromTheme> ListStyle<Color> {
    pub fn new(theme: &Theme) -> Self {
        let normal = Style::new()
            .color_theme(theme, ThemeColor::Foreground)
            .margin(Insets::new(0, 0, 2, 2))
            .padding(Insets::all(2))
            .align(Align::Center);

        let active = normal
            .clone()
            .color_theme(theme, ThemeColor::Foreground)
            .background_theme(theme, ThemeColor::Background)
            .radius_all(4)
            .margin(Insets::new(0, 0, 4, 4))
            .border_theme(theme, 2, ThemeColor::Muted);

        Self { normal, active }
    }
}

pub struct ScrollableList<'a, T, Color: PixelColor> {
    items: &'a [T],
    label: fn(&T) -> &String,
    trailing_label: Option<&'a str>,
    window_size: usize,
    style: ListStyle<Color>,
}

impl<'a, T, Color: PixelColor + FromTheme> ScrollableList<'a, T, Color> {
    pub fn new(
        items: &'a [T],
        theme: &Theme,
        label: fn(&T) -> &String,
        window_size: usize,
    ) -> Self {
        Self {
            items,
            label,
            trailing_label: None,
            window_size,
            style: ListStyle::new(theme),
        }
    }

    pub fn with_trailing(mut self, label: &'a str) -> Self {
        self.trailing_label = Some(label);
        self
    }

    fn total(&self) -> usize {
        self.items.len() + self.trailing_label.is_some() as usize
    }

    fn update_offset(&self, scroll: &mut ScrollState, selected: usize) {
        let total = self.total();
        let max_offset = total.saturating_sub(self.window_size);

        if selected >= scroll.offset + self.window_size {
            scroll.offset = selected - self.window_size + 1;
        } else if selected < scroll.offset {
            scroll.offset = (selected / self.window_size) * self.window_size;
        }

        scroll.offset = scroll.offset.min(max_offset);
    }

    pub fn render<D>(
        &self,
        display: &mut D,
        area: Rectangle,
        scroll: &mut ScrollState,
        selected: usize,
    ) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Color> + OriginDimensions,
        D::Color: FromTheme,
    {
        self.update_offset(scroll, selected);

        let flexbox = Flexbox::new(area, 2i32);
        let allocated = flexbox.vertical(&vec![1; self.window_size]);

        for (index, row_area) in allocated.into_iter().enumerate() {
            let absolute_index = index + scroll.offset;
            let is_selected = absolute_index == selected;

            let text = match self.items.get(absolute_index) {
                Some(item) => (self.label)(item),
                None => self.trailing_label.unwrap_or_default(),
            };

            let (style, font_style) = if is_selected {
                (self.style.active, TextStyle::BoldMedium.value::<D::Color>())
            } else {
                (self.style.normal, TextStyle::Medium.value::<D::Color>())
            };

            let painted_area = style.paint(display, row_area)?;
            style.draw_text(display, painted_area, &text, font_style.font)?;
        }

        Ok(())
    }
}
