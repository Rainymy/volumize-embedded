#![allow(dead_code)]
use embedded_graphics::{
    Drawable, Pixel,
    draw_target::DrawTarget,
    geometry::{Point, Size},
    mono_font::{MonoFont, MonoTextStyle},
    pixelcolor::PixelColor,
    primitives::{CornerRadii, Primitive, PrimitiveStyleBuilder, Rectangle, RoundedRectangle},
    text::{Alignment, Baseline, Text, TextStyleBuilder},
};

use super::Bitmap;

// ---------- Corners (per-corner radius, e.g. border-radius) ----------

#[derive(Debug, Clone, Copy, Default)]
pub struct Corners {
    pub top_left: Size,
    pub top_right: Size,
    pub bottom_left: Size,
    pub bottom_right: Size,
}

impl Corners {
    pub fn new(top_left: u32, top_right: u32, bottom_left: u32, bottom_right: u32) -> Self {
        Self {
            top_left: Size::new(top_left, top_left),
            top_right: Size::new(top_right, top_right),
            bottom_left: Size::new(bottom_left, bottom_left),
            bottom_right: Size::new(bottom_right, bottom_right),
        }
    }
    pub fn all(value: u32) -> Self {
        Self {
            top_left: Size::new(value, value),
            top_right: Size::new(value, value),
            bottom_left: Size::new(value, value),
            bottom_right: Size::new(value, value),
        }
    }

    pub fn is_zero(&self) -> bool {
        let width = self.top_left.width == 0
            && self.top_right.width == 0
            && self.bottom_left.width == 0
            && self.bottom_right.width == 0;
        let height = self.top_left.height == 0
            && self.top_right.height == 0
            && self.bottom_left.height == 0
            && self.bottom_right.height == 0;
        width && height
    }
}

// ---------- Insets (edges, e.g. margin/padding) ----------

#[derive(Debug, Clone, Copy, Default)]
pub struct Insets {
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
    pub left: i32,
}

impl Insets {
    pub fn new(top: i32, right: i32, bottom: i32, left: i32) -> Self {
        Self {
            top,
            right,
            bottom,
            left,
        }
    }
    pub fn all(value: i32) -> Self {
        Self {
            top: value,
            right: value,
            bottom: value,
            left: value,
        }
    }
}

// ---------- Border ----------

#[derive(Debug, Clone, Copy, Default)]
pub struct Border<C: PixelColor> {
    pub width: u32,
    pub color: C,
}

#[derive(Debug, Clone, Copy, Default)]
pub enum Align {
    Start,
    #[default]
    Center,
    End,
}

impl Into<Alignment> for Align {
    fn into(self) -> Alignment {
        match self {
            Align::Start => Alignment::Left,
            Align::Center => Alignment::Center,
            Align::End => Alignment::Right,
        }
    }
}

// ---------- Style ----------
#[derive(Debug, Clone, Copy, Default)]
pub struct Style<Color: PixelColor> {
    pub margin: Insets,
    pub padding: Insets,
    pub background_color: Option<Color>,
    pub foreground_color: Color,
    pub border: Option<Border<Color>>,
    pub radius: Corners,
    pub align: Align,
}

impl<Color: PixelColor> Style<Color> {
    pub fn new(foreground_color: Color) -> Self {
        Self {
            margin: Insets::default(),
            padding: Insets::default(),
            background_color: None,
            foreground_color,
            border: None,
            radius: Corners::default(),
            align: Align::Start,
        }
    }

    pub fn padding(mut self, insets: Insets) -> Self {
        self.padding = insets;
        self
    }

    pub fn padding_all(mut self, value: i32) -> Self {
        self.padding = Insets::all(value);
        self
    }

    pub fn margin(mut self, insets: Insets) -> Self {
        self.margin = insets;
        self
    }

    pub fn margin_all(mut self, value: i32) -> Self {
        self.margin = Insets::all(value);
        self
    }

    pub fn background(mut self, color: Color) -> Self {
        self.background_color = Some(color);
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.foreground_color = color;
        self
    }

    pub fn border(mut self, width: u32, color: Color) -> Self {
        self.border = Some(Border { width, color });
        self
    }

    pub fn radius_all(mut self, r: u32) -> Self {
        self.radius = Corners::all(r);
        self
    }

    pub fn radius(mut self, corners: Corners) -> Self {
        self.radius = corners;
        self
    }

    pub fn align(mut self, a: Align) -> Self {
        self.align = a;
        self
    }

    pub fn align_element_x(&self, parent: Rectangle, source_size: Size, align: Align) -> Rectangle {
        let topleftx = parent.top_left.x;
        let width_offset = parent.size.width as i32 - source_size.width as i32;

        let x = match align {
            Align::Start => topleftx,
            Align::Center => topleftx + width_offset.saturating_div(2),
            Align::End => topleftx + width_offset,
        };

        Rectangle::new(Point::new(x, parent.top_left.y), source_size)
    }

    pub fn align_element_y(&self, parent: Rectangle, source_size: Size, align: Align) -> Rectangle {
        let toplefty = parent.top_left.y;
        let height_offset = parent.size.height as i32 - source_size.height as i32;

        let y = match align {
            Align::Start => toplefty,
            Align::Center => toplefty + height_offset.saturating_div(2),
            Align::End => toplefty + height_offset,
        };

        Rectangle::new(Point::new(parent.top_left.x, y), source_size)
    }

    /// Paints the style onto the given target, within the specified area.
    pub fn paint<D: DrawTarget<Color = Color>>(
        &self,
        target: &mut D,
        area: Rectangle,
    ) -> Result<Rectangle, D::Error> {
        let bordered_area = shrink(area, &self.margin);

        let mut style_builder = PrimitiveStyleBuilder::new();

        if let Some(bg) = self.background_color {
            style_builder = style_builder.fill_color(bg);
        }
        if let Some(border) = &self.border {
            style_builder = style_builder
                .stroke_color(border.color)
                .stroke_width(border.width);
        }
        let primitive_style = style_builder.build();

        if !self.radius.is_zero() {
            let radii = CornerRadii {
                top_right: self.radius.top_right,
                top_left: self.radius.top_left,
                bottom_right: self.radius.bottom_right,
                bottom_left: self.radius.bottom_left,
            };
            RoundedRectangle::new(bordered_area, radii)
                .into_styled(primitive_style)
                .draw(target)?;
        } else {
            bordered_area.into_styled(primitive_style).draw(target)?;
        }

        // content area callers should draw text/children into.
        let border_width = self.border.as_ref().map(|b| b.width as i32).unwrap_or(0);
        let inner_area = shrink(bordered_area, &Insets::all(border_width));
        let content_area = shrink(inner_area, &self.padding);

        Ok(content_area)
    }

    pub fn draw_text<D: DrawTarget<Color = Color>>(
        &self,
        target: &mut D,
        area: Rectangle,
        text: &str,
        font: &MonoFont,
    ) -> Result<(), D::Error> {
        let topleftx = area.top_left.x;
        // Vertically center the text within the area.
        let anchor_y = area.top_left.y + area.size.height as i32 / 2;
        let anchor_x = match self.align {
            Align::Start => topleftx,
            Align::Center => topleftx + area.size.width as i32 / 2,
            Align::End => topleftx + area.size.width as i32,
        };

        let text_style = TextStyleBuilder::new()
            .alignment(self.align.into())
            .baseline(Baseline::Middle)
            .build();

        let char_style = MonoTextStyle::new(font, self.foreground_color);
        let location = Point::new(anchor_x, anchor_y);

        Text::with_text_style(text, location, char_style, text_style).draw(target)?;

        Ok(())
    }

    pub fn draw_bitmap<D: DrawTarget<Color = Color>>(
        &self,
        target: &mut D,
        area: Rectangle,
        bitmap: &Bitmap,
        on_color: Color,
        off_color: Option<Color>,
    ) -> Result<(), D::Error> {
        let img_size = bitmap.size();

        let x = self.align_element_x(area, img_size, self.align);
        let y = self
            .align_element_y(area, img_size, Align::Center)
            .top_left
            .y;

        let origin = Point::new(x.top_left.x, y);

        let pixels = bitmap
            .pixels()
            .filter_map(|(pos, color)| match (color, off_color) {
                (true, _) => Some(Pixel(pos + origin, on_color)),
                (false, Some(off)) => Some(Pixel(pos + origin, off)),
                (false, None) => None, // transparent, skip
            });

        target.draw_iter(pixels)
    }
}

/// Shrinks a rectangle by the given insets on each side.
/// Clamps to zero size instead of panicking if insets exceed the rect.
fn shrink(rect: Rectangle, insets: &Insets) -> Rectangle {
    let x = rect.top_left.x + insets.left;
    let y = rect.top_left.y + insets.top;

    let width = (rect.size.width as i32 - insets.left - insets.right).max(0) as u32;
    let height = (rect.size.height as i32 - insets.top - insets.bottom).max(0) as u32;

    Rectangle::new(Point::new(x, y), Size::new(width, height))
}
