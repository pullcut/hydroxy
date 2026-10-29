//! Colour maths for the colour picker: 8-bit RGBA, HSV, and CSS hex text.

#![allow(
    clippy::float_cmp,
    clippy::many_single_char_names,
    reason = "channels are compared exactly against 0.0 and against the maximum, which is one of them"
)]

/// An sRGB colour with alpha, 8 bits a channel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Rgba {
    /// Red.
    pub r: u8,
    /// Green.
    pub g: u8,
    /// Blue.
    pub b: u8,
    /// Alpha, 255 opaque.
    pub a: u8,
}

/// Hue, saturation and value: the model the picker's area and slider drive.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Hsv {
    /// Hue in degrees, 0–360 (360 is the same hue as 0).
    pub h: f64,
    /// Saturation, 0–1.
    pub s: f64,
    /// Value (brightness), 0–1.
    pub v: f64,
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss, reason = "clamped to 0..=255 first")]
fn byte(unit: f64) -> u8 {
    (unit.clamp(0.0, 1.0) * 255.0).round() as u8
}

impl Rgba {
    /// An opaque colour.
    #[must_use]
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    /// The colour as HSV (alpha is dropped). Greys have hue 0.
    #[must_use]
    pub fn to_hsv(self) -> Hsv {
        let (r, g, b) = (f64::from(self.r) / 255.0, f64::from(self.g) / 255.0, f64::from(self.b) / 255.0);
        let max = r.max(g).max(b);
        let delta = max - r.min(g).min(b);
        let h = if delta == 0.0 {
            0.0
        } else if max == r {
            60.0 * ((g - b) / delta).rem_euclid(6.0)
        } else if max == g {
            60.0 * ((b - r) / delta + 2.0)
        } else {
            60.0 * ((r - g) / delta + 4.0)
        };
        Hsv { h, s: if max == 0.0 { 0.0 } else { delta / max }, v: max }
    }

    /// CSS hex text: `#rrggbb`, or `#rrggbbaa` with `alpha`.
    #[must_use]
    pub fn hex(self, alpha: bool) -> String {
        if alpha {
            format!("#{:02x}{:02x}{:02x}{:02x}", self.r, self.g, self.b, self.a)
        } else {
            format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
        }
    }
}

/// Why text isn't a hex colour.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParseHexError;

impl std::fmt::Display for ParseHexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("expected a hex colour such as #rgb or #rrggbbaa")
    }
}

impl std::error::Error for ParseHexError {}

/// Parses `#rgb`, `#rgba`, `#rrggbb` or `#rrggbbaa`. The `#` is optional.
impl std::str::FromStr for Rgba {
    type Err = ParseHexError;

    fn from_str(text: &str) -> Result<Self, ParseHexError> {
        let digits = text.trim().trim_start_matches('#');
        if !digits.is_ascii() || !digits.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err(ParseHexError);
        }
        let pair = |i: usize| u8::from_str_radix(&digits[i..i + 2], 16).map_err(|_| ParseHexError);
        let short = |i: usize| u8::from_str_radix(&digits[i..=i], 16).map(|n| n * 17).map_err(|_| ParseHexError);
        match digits.len() {
            3 => Ok(Self::rgb(short(0)?, short(1)?, short(2)?)),
            4 => Ok(Self { r: short(0)?, g: short(1)?, b: short(2)?, a: short(3)? }),
            6 => Ok(Self::rgb(pair(0)?, pair(2)?, pair(4)?)),
            8 => Ok(Self { r: pair(0)?, g: pair(2)?, b: pair(4)?, a: pair(6)? }),
            _ => Err(ParseHexError),
        }
    }
}

impl From<Rgba> for Hsv {
    fn from(c: Rgba) -> Self {
        c.to_hsv()
    }
}

impl From<Hsv> for Rgba {
    fn from(c: Hsv) -> Self {
        c.to_rgb()
    }
}

impl Hsv {
    /// The opaque RGB colour for this hue, saturation and value.
    #[must_use]
    pub fn to_rgb(self) -> Rgba {
        let h = self.h.rem_euclid(360.0) / 60.0;
        let (s, v) = (self.s.clamp(0.0, 1.0), self.v.clamp(0.0, 1.0));
        let c = v * s;
        let x = c * (1.0 - (h.rem_euclid(2.0) - 1.0).abs());
        let (r, g, b) = match h {
            h if h < 1.0 => (c, x, 0.0),
            h if h < 2.0 => (x, c, 0.0),
            h if h < 3.0 => (0.0, c, x),
            h if h < 4.0 => (0.0, x, c),
            h if h < 5.0 => (x, 0.0, c),
            _ => (c, 0.0, x),
        };
        let m = v - c;
        Rgba::rgb(byte(r + m), byte(g + m), byte(b + m))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn primaries_and_greys() {
        assert_eq!(Hsv { h: 0.0, s: 1.0, v: 1.0 }.to_rgb(), Rgba::rgb(255, 0, 0));
        assert_eq!(Hsv { h: 120.0, s: 1.0, v: 1.0 }.to_rgb(), Rgba::rgb(0, 255, 0));
        assert_eq!(Hsv { h: 240.0, s: 1.0, v: 1.0 }.to_rgb(), Rgba::rgb(0, 0, 255));
        assert_eq!(Hsv { h: 360.0, s: 1.0, v: 1.0 }.to_rgb(), Rgba::rgb(255, 0, 0));
        assert_eq!(Hsv { h: 200.0, s: 0.0, v: 0.5 }.to_rgb(), Rgba::rgb(128, 128, 128));
        assert_eq!(Rgba::rgb(255, 255, 0).to_hsv(), Hsv { h: 60.0, s: 1.0, v: 1.0 });
        assert_eq!(Rgba::rgb(0, 0, 0).to_hsv(), Hsv { h: 0.0, s: 0.0, v: 0.0 });
    }

    #[test]
    fn rgb_survives_a_trip_through_hsv() {
        for r in (0..=255).step_by(5) {
            for g in (0..=255).step_by(5) {
                for b in (0..=255).step_by(5) {
                    let c = Rgba::rgb(r, g, b);
                    assert_eq!(Rgba::from(Hsv::from(c)), c);
                }
            }
        }
    }

    #[test]
    fn hex_parses_and_prints() {
        assert_eq!("#ff8800".parse::<Rgba>().ok(), Some(Rgba::rgb(255, 136, 0)));
        assert_eq!("F80".parse::<Rgba>().ok(), Some(Rgba::rgb(255, 136, 0)));
        assert_eq!("#f808".parse::<Rgba>().ok(), Some(Rgba { r: 255, g: 136, b: 0, a: 136 }));
        assert_eq!(" #11223380 ".parse::<Rgba>().ok(), Some(Rgba { r: 17, g: 34, b: 51, a: 128 }));
        for bad in ["", "#", "#ff880", "#ff88001", "#gg0000", "#ééé", "red"] {
            assert_eq!(bad.parse::<Rgba>().ok(), None, "{bad}");
        }
        let c = Rgba { r: 1, g: 171, b: 255, a: 128 };
        assert_eq!(c.hex(false), "#01abff");
        assert_eq!(c.hex(true), "#01abff80");
        assert_eq!(c.hex(true).parse(), Ok(c));
    }
}
