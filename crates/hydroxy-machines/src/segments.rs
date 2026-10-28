//! Digit entry shared by the segmented fields ([`date_field`](crate::date_field),
//! [`duration_field`](crate::duration_field)): typing into a segment, and stepping it.

/// The range and width of one numeric segment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Spec {
    /// Lowest value.
    pub min: u32,
    /// Highest value.
    pub max: u32,
    /// Digits when full (2 for a day, 4 for a year).
    pub width: u8,
}

/// The result of typing one digit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Typed {
    /// The segment's new value.
    pub value: u32,
    /// Digits typed into it so far.
    pub digits: u8,
    /// No further digit could fit: move on to the next segment.
    pub advance: bool,
}

/// Type `digit` into a segment that holds `prev` after `prev_digits` typed digits
/// (0 when the user just arrived). A digit that would overflow `max` starts over.
#[must_use]
pub fn type_digit(prev: Option<u32>, prev_digits: u8, digit: u32, spec: Spec) -> Typed {
    let joined =
        prev.filter(|_| prev_digits > 0 && prev_digits < spec.width).map(|p| p * 10 + digit).filter(|v| *v <= spec.max);
    let (value, digits) = joined.map_or((digit, 1), |v| (v, prev_digits + 1));
    Typed { value, digits, advance: digits >= spec.width || value * 10 > spec.max }
}

/// One step up or down, wrapping at the ends. An empty segment takes `start`.
#[must_use]
pub fn step(value: Option<u32>, up: bool, spec: Spec, start: u32) -> u32 {
    match value {
        None => start.clamp(spec.min, spec.max),
        Some(v) if up => {
            if v >= spec.max {
                spec.min
            } else {
                (v + 1).max(spec.min)
            }
        }
        Some(v) => {
            if v <= spec.min {
                spec.max
            } else {
                (v - 1).min(spec.max)
            }
        }
    }
}

/// Zero-padded text for a segment, or `placeholder` dashes while empty.
#[must_use]
pub fn text(value: Option<u32>, width: u8) -> String {
    let w = usize::from(width);
    value.map_or_else(|| "–".repeat(w), |v| format!("{v:0w$}"))
}

/// What a key does inside a segment, whatever the field.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SegmentKey {
    /// A digit to type.
    Digit(u32),
    /// Arrow Up or Down.
    Step {
        /// Up (`true`) or down.
        up: bool,
    },
    /// Home or End.
    Jump {
        /// End (`true`) or Home.
        to_max: bool,
    },
    /// Backspace or Delete.
    Clear,
    /// Arrow Left: the segment before.
    Prev,
    /// Arrow Right: the segment after.
    Next,
    /// Any other character: swallowed, so the input never holds free text.
    Ignore,
}

/// Classify a `KeyboardEvent.key`. `modified` is Ctrl, Meta or Alt held, which
/// leaves shortcuts (copy, reload) alone. `None` means let the browser handle it (Tab).
#[must_use]
pub fn key(key: &str, modified: bool) -> Option<SegmentKey> {
    if modified {
        return None;
    }
    match key {
        "ArrowUp" => Some(SegmentKey::Step { up: true }),
        "ArrowDown" => Some(SegmentKey::Step { up: false }),
        "Home" => Some(SegmentKey::Jump { to_max: false }),
        "End" => Some(SegmentKey::Jump { to_max: true }),
        "Backspace" | "Delete" => Some(SegmentKey::Clear),
        "ArrowLeft" => Some(SegmentKey::Prev),
        "ArrowRight" => Some(SegmentKey::Next),
        k if k.chars().count() == 1 => {
            Some(k.chars().next().and_then(|c| c.to_digit(10)).map_or(SegmentKey::Ignore, SegmentKey::Digit))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DAY: Spec = Spec { min: 1, max: 31, width: 2 };

    #[test]
    fn typing_joins_or_advances() {
        let t = type_digit(None, 0, 3, DAY);
        assert_eq!((t.value, t.digits, t.advance), (3, 1, false));
        let t = type_digit(Some(3), 1, 1, DAY);
        assert_eq!((t.value, t.digits, t.advance), (31, 2, true));
        // 3 then 5 overflows: start over with 5, which can't take another digit.
        let t = type_digit(Some(3), 1, 5, DAY);
        assert_eq!((t.value, t.digits, t.advance), (5, 1, true));
        // Arriving on a filled segment replaces it.
        assert_eq!(type_digit(Some(31), 0, 2, DAY).value, 2);
    }

    #[test]
    fn stepping_wraps() {
        assert_eq!(step(Some(31), true, DAY, 1), 1);
        assert_eq!(step(Some(1), false, DAY, 1), 31);
        assert_eq!(step(None, true, DAY, 15), 15);
        assert_eq!(step(Some(0), true, DAY, 1), 1);
    }

    #[test]
    fn keys() {
        assert_eq!(key("7", false), Some(SegmentKey::Digit(7)));
        assert_eq!(key("a", false), Some(SegmentKey::Ignore));
        assert_eq!(key("ArrowDown", false), Some(SegmentKey::Step { up: false }));
        assert_eq!(key("Tab", false), None);
        assert_eq!(key("7", true), None);
    }

    #[test]
    fn padding() {
        assert_eq!(text(Some(4), 2), "04");
        assert_eq!(text(None, 4), "––––");
    }
}
