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

use alloc::vec::Vec;

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

// ---------- Bitmap ----------

pub struct Bitmap<'a> {
    data: &'a [u8],
    width: u32,
    height: u32,
}

impl<'a> Bitmap<'a> {
    pub fn new(data: &'a [u8], width: u32, height: u32) -> Self {
        Self {
            data,
            width,
            height,
        }
    }

    pub fn size(&self) -> Size {
        Size::new(self.width, self.height)
    }

    fn bytes_per_row(&self) -> u32 {
        (self.width + 7) / 8
    }

    /// true = "on" bit, false = "off" bit.
    fn get(&self, x: u32, y: u32) -> bool {
        let stride = self.bytes_per_row();
        let byte_index = (y * stride + x / 8) as usize;
        let bit_index = 7 - (x % 8); // MSB first
        (self.data[byte_index] >> bit_index) & 1 == 1
    }

    pub fn pixels(&self) -> impl Iterator<Item = (Point, bool)> + '_ {
        let (w, h) = (self.width, self.height);

        (0..h).flat_map(move |y| {
            (0..w).map(move |x| {
                let point = Point::new(x as i32, y as i32);
                let is_on = self.get(x, y);
                (point, is_on)
            })
        })
    }
}

// ---------- Style ----------

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

        let x = match align {
            Align::Start => topleftx,
            Align::Center => topleftx + (parent.size.width - source_size.width) as i32 / 2,
            Align::End => topleftx + (parent.size.width - source_size.width) as i32,
        };

        let new_point = Point::new(x, parent.top_left.y);
        Rectangle::new(new_point, source_size)
    }

    pub fn align_element_y(&self, parent: Rectangle, source_size: Size, align: Align) -> Rectangle {
        let toplefty = parent.top_left.y;

        let y = match align {
            Align::Start => toplefty,
            Align::Center => toplefty + (parent.size.height - source_size.height) as i32 / 2,
            Align::End => toplefty + (parent.size.height - source_size.height) as i32,
        };

        let new_point = Point::new(parent.top_left.x, y);
        Rectangle::new(new_point, source_size)
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

        let x = match self.align {
            Align::Start => area.top_left.x,
            Align::Center => area.top_left.x + (area.size.width as i32 - img_size.width as i32) / 2,
            Align::End => area.top_left.x + area.size.width as i32 - img_size.width as i32,
        };
        let y = area.top_left.y + (area.size.height as i32 - img_size.height as i32) / 2;
        let origin = Point::new(x, y);

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

// ---------------------- FLEXBOX ----------------------

#[derive(Debug, Clone, Copy)]
pub struct Flexbox {
    pub area: Rectangle,
    pub gap: i32,
}

impl Flexbox {
    pub fn new(area: Rectangle, gap: i32) -> Self {
        Self { area, gap }
    }

    /// Splits `area` into `sizes.len()` rectangles stacked top-to-bottom,
    /// with height distributed proportionally to each weight in `sizes`,
    /// separated by `self.gap` pixels.
    pub fn vertical(self, sizes: &[u32]) -> Vec<Rectangle> {
        split(self.area, self.gap, sizes, Axis::Vertical)
    }

    /// Same as `vertical`, but splits left-to-right along width.
    pub fn horizontal(self, sizes: &[u32]) -> Vec<Rectangle> {
        split(self.area, self.gap, sizes, Axis::Horizontal)
    }
}

enum Axis {
    Vertical,
    Horizontal,
}

fn split(area: Rectangle, gap: i32, sizes: &[u32], axis: Axis) -> Vec<Rectangle> {
    if sizes.is_empty() {
        return Vec::new();
    }

    let total_weight: u32 = sizes.iter().sum();
    let n = sizes.len() as i32;
    let total_gap = gap * (n - 1).max(0);

    let (total_extent, fixed_extent, origin_x, origin_y) = match axis {
        Axis::Vertical => (
            area.size.height as i32,
            area.size.width,
            area.top_left.x,
            area.top_left.y,
        ),
        Axis::Horizontal => (
            area.size.width as i32,
            area.size.height,
            area.top_left.x,
            area.top_left.y,
        ),
    };

    let usable = (total_extent - total_gap).max(0) as u32;

    let mut rects = Vec::with_capacity(sizes.len());
    let mut cursor = 0i32;
    let mut cumulative_weight: u32 = 0;
    let mut prev_boundary: u32 = 0;

    for &weight in sizes {
        cumulative_weight += weight;

        let boundary = if total_weight == 0 {
            0
        } else {
            (usable * cumulative_weight) / total_weight
        };

        let extent = boundary - prev_boundary;
        prev_boundary = boundary;

        let rect = match axis {
            Axis::Vertical => Rectangle::new(
                Point::new(origin_x, origin_y + cursor),
                Size::new(fixed_extent, extent),
            ),
            Axis::Horizontal => Rectangle::new(
                Point::new(origin_x + cursor, origin_y),
                Size::new(extent, fixed_extent),
            ),
        };

        rects.push(rect);
        cursor += extent as i32 + gap;
    }

    rects
}
