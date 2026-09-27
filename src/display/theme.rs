use embedded_graphics::pixelcolor::{BinaryColor, Rgb565, Rgb888, RgbColor};

#[derive(Debug, Clone, Copy)]
pub struct Theme {
    background: Rgb888,
    foreground: Rgb888,
    accent: Rgb888,
    muted: Rgb888,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeColor {
    Background,
    Foreground,
    Accent,
    Muted,
}

impl Theme {
    pub const fn base() -> Self {
        Self {
            background: Rgb888::new(0, 0, 0),
            foreground: Rgb888::new(255, 255, 255),
            accent: Rgb888::new(255, 255, 255),
            muted: Rgb888::new(0, 0, 0),
        }
    }

    fn resolve(&self, token: ThemeColor) -> Rgb888 {
        match token {
            ThemeColor::Background => self.background,
            ThemeColor::Foreground => self.foreground,
            ThemeColor::Accent => self.accent,
            ThemeColor::Muted => self.muted,
        }
    }

    /// pct: -100..=100, negative = darker, positive = brighter.
    #[allow(dead_code)]
    pub fn adjusted(&self, pct: i8) -> Self {
        let scale = |c: Rgb888| -> Rgb888 {
            let f = |v: u8| -> u8 {
                let v = v as i32;
                let delta = v * pct as i32 / 100;
                (v + delta).clamp(0, 255) as u8
            };
            Rgb888::new(f(c.r()), f(c.g()), f(c.b()))
        };
        Self {
            background: scale(self.background),
            foreground: scale(self.foreground),
            accent: scale(self.accent),
            muted: scale(self.muted),
        }
    }
}

pub trait FromTheme: Sized {
    fn from_theme(theme: &Theme, token: ThemeColor) -> Self;
}

impl FromTheme for Rgb565 {
    fn from_theme(theme: &Theme, token: ThemeColor) -> Self {
        let c = theme.resolve(token);
        Rgb565::new(c.r() >> 3, c.g() >> 2, c.b() >> 3)
    }
}

impl FromTheme for BinaryColor {
    fn from_theme(theme: &Theme, token: ThemeColor) -> Self {
        let c = theme.resolve(token);
        let lum = (c.r() as u16 * 30 + c.g() as u16 * 59 + c.b() as u16 * 11) / 100;
        if lum > 128 {
            BinaryColor::On
        } else {
            BinaryColor::Off
        }
    }
}
