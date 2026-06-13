use ratatui::style::Color;

/// First-class bundle of accent/dim/text colors for the accent widget family.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AccentColors {
    pub accent: Color,
    pub dim: Color,
    pub text: Color,   // main / active text color
}

impl AccentColors {
    pub fn new(accent: Color, dim: Color, text: Color) -> Self {
        Self { accent, dim, text }
    }

    pub fn from_accent_dim_text(accent: Color, dim: Color, text: Color) -> Self {
        Self::new(accent, dim, text)
    }

    /// Queries the system theme and DWM accent colors dynamically.
    pub fn query_system() -> Self {
        let dark_mode = crate::backend::sys_info::query_dark_mode();
        let (r, g, b) = crate::backend::sys_info::query_accent_color();
        let accent = Color::Rgb(r, g, b);
        
        let (dim, text) = if dark_mode {
            let dim_r = (r as f32 * 0.35) as u8;
            let dim_g = (g as f32 * 0.35) as u8;
            let dim_b = (b as f32 * 0.35) as u8;
            (Color::Rgb(dim_r, dim_g, dim_b), Color::Gray)
        } else {
            let dim_r = (r as f32 * 0.7) as u8;
            let dim_g = (g as f32 * 0.7) as u8;
            let dim_b = (b as f32 * 0.7) as u8;
            (Color::Rgb(dim_r, dim_g, dim_b), Color::Black)
        };
        
        Self { accent, dim, text }
    }

    /// Constructs a standard theme using a custom accent color.
    pub fn calculate_from_accent(accent: Color, is_dark_mode: bool) -> Self {
        let (r, g, b) = match accent {
            Color::Rgb(r, g, b) => (r, g, b),
            Color::Cyan => (0, 245, 255),
            Color::Red => (255, 0, 0),
            Color::Green => (0, 255, 0),
            Color::Blue => (0, 0, 255),
            Color::Yellow => (255, 255, 0),
            Color::Magenta => (255, 0, 255),
            _ => (0, 245, 255),
        };
        
        let (dim, text) = if is_dark_mode {
            let dim_r = (r as f32 * 0.35) as u8;
            let dim_g = (g as f32 * 0.35) as u8;
            let dim_b = (b as f32 * 0.35) as u8;
            (Color::Rgb(dim_r, dim_g, dim_b), Color::Gray)
        } else {
            let dim_r = (r as f32 * 0.7) as u8;
            let dim_g = (g as f32 * 0.7) as u8;
            let dim_b = (b as f32 * 0.7) as u8;
            (Color::Rgb(dim_r, dim_g, dim_b), Color::Black)
        };
        
        Self { accent, dim, text }
    }
}

pub struct AccentTheme;

impl AccentTheme {
    /// Fetches the current system theme and returns an active `AccentColors` bundle.
    pub fn current() -> AccentColors {
        AccentColors::query_system()
    }

    /// Returns a fallback dark-mode theme using standard apps Cyan.
    pub fn default_dark() -> AccentColors {
        AccentColors {
            accent: Color::Rgb(0, 245, 255),
            dim: Color::Rgb(0, 80, 85),
            text: Color::Gray,
        }
    }

    /// Returns a fallback light-mode theme using standard apps Cyan.
    pub fn default_light() -> AccentColors {
        AccentColors {
            accent: Color::Rgb(0, 180, 200),
            dim: Color::Rgb(180, 230, 240),
            text: Color::Black,
        }
    }
}
