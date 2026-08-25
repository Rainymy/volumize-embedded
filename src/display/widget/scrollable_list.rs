use alloc::{
    string::{String, ToString},
    vec,
};
use embedded_graphics::{pixelcolor::BinaryColor, prelude::*, primitives::Rectangle};

use crate::display::{
    style::{Align, Flexbox, Insets, Style},
    text_style::TextStyle,
};

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScrollState {
    offset: usize,
}

pub struct ListStyle {
    pub normal: Style<BinaryColor>,
    pub active: Style<BinaryColor>,
}

impl Default for ListStyle {
    fn default() -> Self {
        let normal = Style::new(BinaryColor::On)
            .color(BinaryColor::On)
            .margin(Insets::new(0, 0, 2, 2))
            .padding(Insets::all(2))
            .align(Align::Center);

        let active = normal
            .clone()
            .color(BinaryColor::Off)
            .background(BinaryColor::On)
            .radius_all(4)
            .margin(Insets::new(0, 0, 4, 4))
            .border(2, BinaryColor::On);

        Self { normal, active }
    }
}

pub struct ScrollableList<'a, T> {
    items: &'a [T],
    label: fn(&T) -> String,
    trailing_label: Option<&'a str>,
    window_size: usize,
    style: ListStyle,
}

impl<'a, T> ScrollableList<'a, T> {
    pub fn new(items: &'a [T], label: fn(&T) -> String, window_size: usize) -> Self {
        Self {
            items,
            label,
            trailing_label: None,
            window_size,
            style: ListStyle::default(),
        }
    }

    pub fn with_trailing(mut self, label: &'a str) -> Self {
        self.trailing_label = Some(label);
        self
    }

    pub fn with_style(mut self, style: ListStyle) -> Self {
        self.style = style;
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
        D: DrawTarget<Color = BinaryColor> + OriginDimensions,
    {
        self.update_offset(scroll, selected);
        let offset = scroll.offset;

        let flexbox = Flexbox::new(area, 2i32);
        let allocated = flexbox.vertical(&vec![1; self.window_size]);

        let total = self.total();

        for (index, row_area) in allocated.into_iter().enumerate() {
            let absolute_index = index + offset;
            if absolute_index >= total {
                break;
            }

            let is_selected = absolute_index == selected;
            let font_style = if is_selected {
                TextStyle::BoldMedium.value()
            } else {
                TextStyle::Medium.value()
            };

            let text: String = if absolute_index < self.items.len() {
                (self.label)(&self.items.get(absolute_index).unwrap())
            } else {
                self.trailing_label.unwrap_or_default().to_string()
            };

            if is_selected {
                let painted_area = self.style.active.paint(display, row_area)?;
                self.style
                    .active
                    .draw_text(display, painted_area, &text, font_style.font)?;
            } else {
                self.style
                    .normal
                    .draw_text(display, row_area, &text, font_style.font)?;
            }
        }

        Ok(())
    }
}
