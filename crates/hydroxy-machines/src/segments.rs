//! Digit entry shared by the segmented fields ([`date_field`](crate::date_field),
//! [`duration_field`](crate::duration_field)): typing into a segment, and stepping it.

use crate::Modifiers;
use crate::list::Direction;

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
pub fn step(value: Option<u32>, direction: Direction, spec: Spec, start: u32) -> u32 {
    match value {
        None => start.clamp(spec.min, spec.max),
        Some(v) if direction == Direction::Next => {
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SegmentKey {
    /// A digit to type.
    Digit(u32),
    /// Arrow Up or Down.
    Step {
        /// Up ([`Direction::Next`]) or down.
        direction: Direction,
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
pub fn key(key: &str, mods: Modifiers) -> Option<SegmentKey> {
    if mods.ctrl || mods.meta || mods.alt {
        return None;
    }
    match key {
        "ArrowUp" => Some(SegmentKey::Step { direction: Direction::Next }),
        "ArrowDown" => Some(SegmentKey::Step { direction: Direction::Prev }),
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
        assert_eq!(step(Some(31), Direction::Next, DAY, 1), 1);
        assert_eq!(step(Some(1), Direction::Prev, DAY, 1), 31);
        assert_eq!(step(None, Direction::Next, DAY, 15), 15);
        assert_eq!(step(Some(0), Direction::Next, DAY, 1), 1);
    }

    #[test]
    fn key_maps_digits_arrows_and_ignores_letters() {
        assert_eq!(key("7", Modifiers::NONE), Some(SegmentKey::Digit(7)));
        assert_eq!(key("a", Modifiers::NONE), Some(SegmentKey::Ignore));
        assert_eq!(key("ArrowDown", Modifiers::NONE), Some(SegmentKey::Step { direction: Direction::Prev }));
        assert_eq!(key("Tab", Modifiers::NONE), None);
        assert_eq!(key("7", Modifiers::CTRL), None);
    }

    #[test]
    fn text_zero_pads_or_shows_dashes() {
        assert_eq!(text(Some(4), 2), "04");
        assert_eq!(text(None, 4), "––––");
    }
}
