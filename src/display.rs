use ufmt::uWrite;

#[macro_export]
macro_rules! log {
    ($serial:expr, $($arg:tt)*) => {{
        let _ = ufmt::uwrite!($serial, $($arg)*);
    }};
}

// RenderDisplay<const N: usize = 12>
pub trait RenderDisplay {
    fn render<W: uWrite>(&mut self, serial: &mut W, value: i32, button_pressed: bool) -> Result<(), u16>;
}

#[allow(unused)]
pub fn render<D, W>(
    display: &mut D,
    serial: &mut W,
    _value: i32,
    _button_pressed: bool,
) -> Result<(), u16> where
    D: RenderDisplay,
    W: ufmt::uWrite,
{
    display.render(serial, _value, _button_pressed)
}