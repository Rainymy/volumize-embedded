use embedded_graphics::geometry::{Point, Size};

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
