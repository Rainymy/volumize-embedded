#![allow(dead_code)]
use embedded_graphics::{
    mono_font::{MonoTextStyle, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
};

pub enum TextStyle {
    Small,
    Medium,
    Large,
}

impl TextStyle {
    pub const fn value(&self) -> MonoTextStyle<'_, BinaryColor> {
        use embedded_graphics::mono_font::ascii::{FONT_6X10, FONT_8X13, FONT_9X15};

        match self {
            TextStyle::Small => MonoTextStyleBuilder::new()
                .font(&FONT_6X10)
                .text_color(BinaryColor::On)
                .background_color(BinaryColor::Off)
                .build(),
            TextStyle::Medium => MonoTextStyleBuilder::new()
                .font(&FONT_8X13)
                .text_color(BinaryColor::On)
                .background_color(BinaryColor::Off)
                .build(),
            TextStyle::Large => MonoTextStyleBuilder::new()
                .font(&FONT_9X15)
                .text_color(BinaryColor::On)
                .background_color(BinaryColor::Off)
                .build(),
        }
    }
}
