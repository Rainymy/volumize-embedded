#![allow(dead_code)]

use alloc::vec::Vec;
use embedded_graphics::{
    geometry::{Point, Size},
    primitives::Rectangle,
};

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
        return Vec::from([area]);
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
