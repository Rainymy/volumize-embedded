#![allow(dead_code)]
use embedded_graphics::{
    mono_font::{MonoTextStyle, MonoTextStyleBuilder},
    pixelcolor::PixelColor,
};

use crate::display::theme::{FromTheme, Theme, ThemeColor};

pub enum TextStyle {
    Small,
    Medium,
    BoldMedium,
    Large,
}

impl TextStyle {
    pub fn value<'a, Color: PixelColor + FromTheme>(&self) -> MonoTextStyle<'a, Color> {
        use embedded_graphics::mono_font::ascii::{
            FONT_6X10, FONT_8X13, FONT_8X13_BOLD, FONT_9X15,
        };

        let base = Theme::base();

        match self {
            TextStyle::Small => MonoTextStyleBuilder::new()
                .font(&FONT_6X10)
                .text_color(FromTheme::from_theme(&base, ThemeColor::Foreground))
                .background_color(FromTheme::from_theme(&base, ThemeColor::Background))
                .build(),
            TextStyle::Medium => MonoTextStyleBuilder::new()
                .font(&FONT_8X13)
                .text_color(FromTheme::from_theme(&base, ThemeColor::Foreground))
                .background_color(FromTheme::from_theme(&base, ThemeColor::Background))
                .build(),
            TextStyle::BoldMedium => MonoTextStyleBuilder::new()
                .font(&FONT_8X13_BOLD)
                .text_color(FromTheme::from_theme(&base, ThemeColor::Foreground))
                .background_color(FromTheme::from_theme(&base, ThemeColor::Background))
                .build(),
            TextStyle::Large => MonoTextStyleBuilder::new()
                .font(&FONT_9X15)
                .text_color(FromTheme::from_theme(&base, ThemeColor::Foreground))
                .background_color(FromTheme::from_theme(&base, ThemeColor::Background))
                .build(),
        }
    }
}
