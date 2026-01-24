/// A rich Color model with helper conversion methods.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub const WHITE: Self = Self {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };
    pub const BLACK: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };
    pub const RED: Self = Self {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };
    pub const GREEN: Self = Self {
        r: 0.0,
        g: 1.0,
        b: 0.0,
        a: 1.0,
    };
    pub const BLUE: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 1.0,
        a: 1.0,
    };
    pub const YELLOW: Self = Self {
        r: 1.0,
        g: 1.0,
        b: 0.0,
        a: 1.0,
    };
    pub const CYAN: Self = Self {
        r: 0.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };
    pub const MAGENTA: Self = Self {
        r: 1.0,
        g: 0.0,
        b: 1.0,
        a: 1.0,
    };
    pub const ORANGE: Self = Self {
        r: 1.0,
        g: 0.5,
        b: 0.0,
        a: 1.0,
    };
    pub const GRAY: Self = Self {
        r: 0.5,
        g: 0.5,
        b: 0.5,
        a: 1.0,
    };
    pub const TRANSPARENT: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 0.0,
    };

    /// Creates a color from RGBA components (0.0-1.0 range).
    #[inline]
    pub fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    /// Creates an opaque color from RGB components (0.0-1.0 range).
    #[inline]
    pub fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 1.0 }
    }

    /// Creates a color from RGBA bytes (0-255 range).
    #[inline]
    pub fn from_rgba8(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self {
            r: r as f32 / 255.0,
            g: g as f32 / 255.0,
            b: b as f32 / 255.0,
            a: a as f32 / 255.0,
        }
    }

    /// Creates an opaque color from RGB bytes (0-255 range).
    #[inline]
    pub fn from_rgb8(r: u8, g: u8, b: u8) -> Self {
        Self::from_rgba8(r, g, b, 255)
    }

    /// Creates a color from a hex value (0xRRGGBB or 0xRRGGBBAA).
    ///
    /// # Example
    /// ```ignore
    /// let red = Color::from_hex(0xFF0000);
    /// let semi_transparent_blue = Color::from_hex(0x0000FF80);
    /// ```
    pub fn from_hex(hex: u32) -> Self {
        if hex > 0xFFFFFF {
            // Has alpha
            Self::from_rgba8(
                ((hex >> 24) & 0xFF) as u8,
                ((hex >> 16) & 0xFF) as u8,
                ((hex >> 8) & 0xFF) as u8,
                (hex & 0xFF) as u8,
            )
        } else {
            // No alpha, assume opaque
            Self::from_rgb8(
                ((hex >> 16) & 0xFF) as u8,
                ((hex >> 8) & 0xFF) as u8,
                (hex & 0xFF) as u8,
            )
        }
    }

    /// Creates a random opaque color.
    pub fn random() -> Self {
        Self {
            r: rand::random(),
            g: rand::random(),
            b: rand::random(),
            a: 1.0,
        }
    }

    /// Returns a copy of this color with a different alpha value.
    #[must_use]
    pub fn with_alpha(self, a: f32) -> Self {
        Self { a, ..self }
    }

    /// Linearly interpolate between two colors.
    #[must_use]
    pub fn lerp(self, other: Self, t: f32) -> Self {
        Self {
            r: self.r + (other.r - self.r) * t,
            g: self.g + (other.g - self.g) * t,
            b: self.b + (other.b - self.b) * t,
            a: self.a + (other.a - self.a) * t,
        }
    }

    /// Returns a brightened version of this color.
    #[must_use]
    pub fn brighten(self, amount: f32) -> Self {
        Self {
            r: (self.r + amount).clamp(0.0, 1.0),
            g: (self.g + amount).clamp(0.0, 1.0),
            b: (self.b + amount).clamp(0.0, 1.0),
            a: self.a,
        }
    }

    /// Returns a darkened version of this color.
    #[must_use]
    pub fn darken(self, amount: f32) -> Self {
        self.brighten(-amount)
    }
}

impl From<[f32; 4]> for Color {
    fn from(arr: [f32; 4]) -> Self {
        Self {
            r: arr[0],
            g: arr[1],
            b: arr[2],
            a: arr[3],
        }
    }
}

impl From<Color> for [f32; 4] {
    fn from(c: Color) -> Self {
        [c.r, c.g, c.b, c.a]
    }
}
