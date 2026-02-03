use std::env;

use ratatui::style::Color;
use tracing::warn;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeMode {
    Dark,
    Light,
}

#[derive(Debug, Clone, Copy)]
pub struct Palette {
    pub bg: Color,
    pub surface: Color,
    pub text: Color,
    pub text_muted: Color,
    pub text_dim: Color,
    pub border: Color,
    pub accent: Color,
    pub accent_alt: Color,
    pub success: Color,
    pub warning: Color,
    pub error: Color,
    pub selection_bg: Color,
    pub selection_fg: Color,
    pub badge_bg: Color,
    pub badge_fg: Color,
    pub input_bg: Color,
    pub input_fg: Color,
    pub cursor: Color,
}

#[derive(Debug, Clone, Copy)]
pub struct Theme {
    pub mode: ThemeMode,
    pub palette: Palette,
}

impl Theme {
    pub fn dark() -> Self {
        Self {
            mode: ThemeMode::Dark,
            palette: Palette {
                bg: Color::Rgb(0x0B, 0x11, 0x18),
                surface: Color::Rgb(0x15, 0x1D, 0x27),
                text: Color::Rgb(0xE6, 0xED, 0xF3),
                text_muted: Color::Rgb(0xB7, 0xC0, 0xCC),
                text_dim: Color::Rgb(0x8A, 0x94, 0xA6),
                border: Color::Rgb(0x2B, 0x35, 0x45),
                accent: Color::Rgb(0x5C, 0xC8, 0xFF),
                accent_alt: Color::Rgb(0x8F, 0xA8, 0xFF),
                success: Color::Rgb(0x6B, 0xD2, 0x8A),
                warning: Color::Rgb(0xF5, 0xC8, 0x6A),
                error: Color::Rgb(0xFF, 0x8A, 0x8A),
                selection_bg: Color::Rgb(0x27, 0x4A, 0x7A),
                selection_fg: Color::Rgb(0xF8, 0xFA, 0xFF),
                badge_bg: Color::Rgb(0x1F, 0x2A, 0x37),
                badge_fg: Color::Rgb(0xE6, 0xED, 0xF3),
                input_bg: Color::Rgb(0x1A, 0x23, 0x30),
                input_fg: Color::Rgb(0xF6, 0xF8, 0xFA),
                cursor: Color::Rgb(0x5C, 0xC8, 0xFF),
            },
        }
    }

    pub fn light() -> Self {
        Self {
            mode: ThemeMode::Light,
            palette: Palette {
                bg: Color::Rgb(0xF8, 0xFA, 0xFC),
                surface: Color::Rgb(0xEE, 0xF2, 0xF7),
                text: Color::Rgb(0x0F, 0x17, 0x2A),
                text_muted: Color::Rgb(0x47, 0x55, 0x69),
                text_dim: Color::Rgb(0x64, 0x74, 0x8B),
                border: Color::Rgb(0xCB, 0xD5, 0xE1),
                accent: Color::Rgb(0x0B, 0x76, 0xD1),
                accent_alt: Color::Rgb(0x25, 0x63, 0xEB),
                success: Color::Rgb(0x15, 0x80, 0x3D),
                warning: Color::Rgb(0xB4, 0x53, 0x09),
                error: Color::Rgb(0xB4, 0x23, 0x18),
                selection_bg: Color::Rgb(0xD6, 0xE4, 0xFF),
                selection_fg: Color::Rgb(0x0B, 0x1B, 0x2B),
                badge_bg: Color::Rgb(0xDC, 0xE7, 0xF3),
                badge_fg: Color::Rgb(0x0F, 0x17, 0x2A),
                input_bg: Color::Rgb(0xE2, 0xE8, 0xF0),
                input_fg: Color::Rgb(0x0F, 0x17, 0x2A),
                cursor: Color::Rgb(0x0B, 0x76, 0xD1),
            },
        }
    }

    pub fn from_env() -> Self {
        match env::var("KATHA_THEME") {
            Ok(value) => match value.trim().to_ascii_lowercase().as_str() {
                "light" => Self::light(),
                "dark" | "" => Self::dark(),
                other => {
                    warn!("Unknown KATHA_THEME value: {}", other);
                    Self::dark()
                }
            },
            Err(_) => Self::dark(),
        }
    }

    pub fn toggle(self) -> Self {
        match self.mode {
            ThemeMode::Dark => Self::light(),
            ThemeMode::Light => Self::dark(),
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::dark()
    }
}
