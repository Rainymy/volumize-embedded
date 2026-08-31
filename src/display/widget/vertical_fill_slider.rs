use embedded_graphics::{
    draw_target::DrawTarget,
    geometry::{Point, Size},
    mono_font::MonoTextStyle,
    pixelcolor::BinaryColor,
    primitives::Rectangle,
    text::{Alignment, Text, renderer::CharacterStyle},
};

use crate::display::{
    Percentage,
    style::{Align, Corners, Style},
};

#[derive(Debug, Clone, Copy)]
pub enum SliderAlign {
    Vertical,
    Horizontal,
}

pub struct VerticalSlider {
    pub outer_style: Style<BinaryColor>,
    pub inner_style_full: Style<BinaryColor>,
    pub inner_style_partial: Style<BinaryColor>,
}

impl Default for VerticalSlider {
    fn default() -> Self {
        let outer_style = Style::new(BinaryColor::On)
            .margin_all(1)
            .padding_all(1)
            .border(1, BinaryColor::On)
            .background(BinaryColor::Off)
            .radius_all(3)
            .align(Align::Center);

        let inner_style_full = Style::new(BinaryColor::On)
            .padding_all(1)
            .background(BinaryColor::On)
            .border(1, BinaryColor::On)
            .radius_all(3);

        let inner_style_partial = Style::new(BinaryColor::On)
            .padding_all(1)
            .background(BinaryColor::On)
            .border(1, BinaryColor::On)
            .radius(Corners::new(0, 0, 3, 3));

        Self {
            outer_style,
            inner_style_full,
            inner_style_partial,
        }
    }
}

impl VerticalSlider {
    pub fn render<D>(
        &self,
        display: &mut D,
        area: Rectangle,
        percentage: &Percentage,
        align: SliderAlign,
    ) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = BinaryColor>,
    {
        let area = {
            match align {
                SliderAlign::Vertical => {
                    self.outer_style
                        .align_element_x(area, area.size, Align::Center)
                }
                SliderAlign::Horizontal => {
                    self.outer_style
                        .align_element_y(area, area.size, Align::Center)
                }
            }
        };

        let working_area = self.outer_style.paint(display, area)?;

        let inner_style = if percentage.is_max() {
            &self.inner_style_full
        } else {
            &self.inner_style_partial
        };

        let aligned_area = match align {
            SliderAlign::Vertical => {
                let fill_size = Size::new(
                    working_area.size.width,
                    (working_area.size.height as f32 * percentage.to_float()) as u32,
                );

                inner_style.align_element_y(working_area, fill_size, Align::End)
            }
            SliderAlign::Horizontal => {
                let fill_size = Size::new(
                    (working_area.size.width as f32 * percentage.to_float()) as u32,
                    working_area.size.height,
                );

                inner_style.align_element_x(working_area, fill_size, Align::End)
            }
        };

        inner_style.paint(display, aligned_area)?;

        Ok(())
    }

    pub fn render_labeled<D>(
        &self,
        display: &mut D,
        area: Rectangle,
        percentage: &Percentage,
        align: SliderAlign,
        label: &str,
        label_style: MonoTextStyle<BinaryColor>,
    ) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = BinaryColor>,
    {
        let area = {
            match align {
                SliderAlign::Vertical => {
                    self.outer_style
                        .align_element_x(area, area.size, Align::Center)
                }
                SliderAlign::Horizontal => {
                    self.outer_style
                        .align_element_y(area, area.size, Align::Center)
                }
            }
        };

        let working_area = self.outer_style.paint(display, area)?;

        let inner_style = if percentage.is_max() {
            &self.inner_style_full
        } else {
            &self.inner_style_partial
        };

        let aligned_area = match align {
            SliderAlign::Vertical => {
                let fill_size = Size::new(
                    working_area.size.width,
                    (working_area.size.height as f32 * percentage.to_float()) as u32,
                );

                inner_style.align_element_y(working_area, fill_size, Align::End)
            }
            SliderAlign::Horizontal => {
                let fill_size = Size::new(
                    (working_area.size.width as f32 * percentage.to_float()) as u32,
                    working_area.size.height,
                );

                inner_style.align_element_x(working_area, fill_size, Align::End)
            }
        };

        inner_style.paint(display, aligned_area)?;

        self.render_label_with_inversion(display, working_area, aligned_area, label, label_style)?;

        Ok(())
    }

    fn render_label_with_inversion<D>(
        &self,
        display: &mut D,
        working_area: Rectangle,
        fill_area: Rectangle,
        text: &str,
        style: MonoTextStyle<BinaryColor>,
    ) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = BinaryColor>,
    {
        use embedded_graphics::Drawable;
        use embedded_graphics::draw_target::DrawTargetExt;
        use embedded_graphics::geometry::Dimensions;

        let center = working_area.center();

        let probe = Text::with_alignment(text, center, style, Alignment::Center);
        let label_bbox = probe.bounding_box();

        let mut inverted_style = style.clone();
        inverted_style.set_text_color(style.text_color.map(|c| c.invert()));
        // inverted_style.set_background_color(style.text_color.map(|c| c.invert()));

        if let Some(covered) = clamp_intersection(label_bbox, fill_area) {
            Text::with_alignment(text, center, inverted_style, Alignment::Center)
                .draw(&mut display.clipped(&covered))?;
        }

        for region in subtract_rect(label_bbox, fill_area) {
            Text::with_alignment(text, center, style, Alignment::Center)
                .draw(&mut display.clipped(&region))?;
        }

        Ok(())
    }
}

fn clamp_intersection(a: Rectangle, b: Rectangle) -> Option<Rectangle> {
    let a_right = a.top_left.x.saturating_add(a.size.width.cast_signed());
    let a_bottom = a.top_left.y + a.size.height as i32;
    let b_right = b.top_left.x + b.size.width as i32;
    let b_bottom = b.top_left.y + b.size.height as i32;

    let left = a.top_left.x.max(b.top_left.x);
    let top = a.top_left.y.max(b.top_left.y);
    let right = a_right.min(b_right);
    let bottom = a_bottom.min(b_bottom);

    if right <= left || bottom <= top {
        None // no real overlap — don't produce a degenerate/garbage rect
    } else {
        Some(Rectangle::new(
            Point::new(left, top),
            Size::new((right - left) as u32, (bottom - top) as u32),
        ))
    }
}

use alloc::{vec, vec::Vec};
fn subtract_rect(rect: Rectangle, cutout: Rectangle) -> Vec<Rectangle> {
    let Some(overlap) = clamp_intersection(rect, cutout) else {
        return vec![rect];
    };
    let mut pieces = Vec::with_capacity(2);

    // Part of `rect` above the overlap.
    if overlap.top_left.y > rect.top_left.y {
        pieces.push(Rectangle::new(
            rect.top_left,
            Size::new(
                rect.size.width,
                (overlap.top_left.y - rect.top_left.y) as u32,
            ),
        ));
    }

    // Part of `rect` below the overlap.
    let rect_bottom = rect.top_left.y + rect.size.height as i32;
    let overlap_bottom = overlap.top_left.y + overlap.size.height as i32;
    if overlap_bottom < rect_bottom {
        pieces.push(Rectangle::new(
            Point::new(rect.top_left.x, overlap_bottom),
            Size::new(rect.size.width, (rect_bottom - overlap_bottom) as u32),
        ));
    }

    pieces
}
