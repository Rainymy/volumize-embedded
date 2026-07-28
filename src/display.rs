use ufmt::uWrite;

// RenderDisplay<const N: usize = 12>
pub trait RenderDisplay {
    fn render<W: uWrite>(
        &mut self,
        serial: &mut W,
        value: u32,
        button_pressed: bool,
    ) -> Result<(), u16>;
}

#[allow(unused)]
pub fn render<D, W>(
    display: &mut D,
    serial: &mut W,
    _value: u32,
    _button_pressed: bool,
) -> Result<(), u16>
where
    D: RenderDisplay,
    W: uWrite,
{
    display.render(serial, _value, _button_pressed)
}
