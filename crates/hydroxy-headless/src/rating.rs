//! Rating connector. Anatomy: root, star (each a `role="radio"` button),
//! half (left-half hit area, only with half stars), icon and a hidden input.
//! Every part carries `data-scope="rating"` and its `data-part`.
//!
//! Interactive ratings are radio groups with a roving tab stop. Read-only
//! ratings are a single `role="img"` with a "Rated 3.5 out of 5" label.

pub use hydroxy_machines::rating::*;

/// `data-scope` for every part.
pub const SCOPE: &str = "rating";

/// Attributes for the root.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RootAttrs {
    /// `"radiogroup"`, or `"img"` when read-only.
    pub role: &'static str,
    /// The given label, or for read-only ratings "Rated 3.5 out of 5".
    pub aria_label: String,
    /// `aria-disabled`.
    pub aria_disabled: bool,
}

/// Attributes for one star.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StarAttrs {
    /// Always `"radio"`.
    pub role: &'static str,
    /// This star holds the value.
    pub aria_checked: bool,
    /// "3 stars", or "2.5 stars" on the star holding a half value.
    pub aria_label: String,
    /// Roving tab stop: the checked star, else the first.
    pub tabindex: i32,
    /// Native `disabled`.
    pub disabled: bool,
    /// `"full"`, `"half"` or `"empty"`, from the hover preview or the value.
    pub data_state: &'static str,
}

fn stars(v: f64) -> String {
    if (v - 1.0).abs() < f64::EPSILON { "1 star".into() } else { format!("{v} stars") }
}

/// Attributes for the root.
#[must_use]
pub fn root_attrs(state: &RatingState, input: &RatingInput, label: &str) -> RootAttrs {
    RootAttrs {
        role: if input.read_only { "img" } else { "radiogroup" },
        aria_label: if input.read_only { format!("Rated {} out of {}", state.value, input.count) } else { label.into() },
        aria_disabled: input.disabled,
    }
}

/// How full the 1-based star `index` is when showing `v`.
#[must_use]
pub fn fill(v: f64, index: usize) -> &'static str {
    let star = index_f64(index);
    if v >= star {
        "full"
    } else if v >= star - 0.5 {
        "half"
    } else {
        "empty"
    }
}

#[allow(clippy::cast_precision_loss, reason = "star counts are tiny")]
fn index_f64(index: usize) -> f64 {
    index as f64
}

/// Attributes for the 1-based star `index`.
#[must_use]
pub fn star_attrs(state: &RatingState, input: &RatingInput, index: usize) -> StarAttrs {
    let holds = state.value > 0.0 && star_of(state.value) == index;
    StarAttrs {
        role: "radio",
        aria_checked: holds,
        aria_label: stars(if holds { state.value } else { index_f64(index) }),
        tabindex: if input.disabled || star_of(state.value) != index { -1 } else { 0 },
        disabled: input.disabled,
        data_state: fill(state.display(), index),
    }
}

/// The value a click picks: the left half of a star with half stars, else the whole star.
#[must_use]
pub fn pick(index: usize, left_half: bool) -> f64 {
    index_f64(index) - if left_half { 0.5 } else { 0.0 }
}

/// Map a `KeyboardEvent.key` to a machine event. `Some` means: prevent default.
#[must_use]
pub fn keydown(key: &str) -> Option<RatingEvent> {
    let key = match key {
        "ArrowRight" | "ArrowUp" => RatingKey::Increment,
        "ArrowLeft" | "ArrowDown" => RatingKey::Decrement,
        "Home" => RatingKey::First,
        "End" => RatingKey::Last,
        _ => return None,
    };
    Some(RatingEvent::Key(key))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state(value: f64) -> RatingState {
        RatingState { value, hover: None, controlled: false }
    }

    #[test]
    fn fill_states() {
        assert_eq!(fill(2.5, 2), "full");
        assert_eq!(fill(2.5, 3), "half");
        assert_eq!(fill(2.5, 4), "empty");
    }

    #[test]
    fn the_holding_star_is_checked_and_tabbable() {
        let input = RatingInput { allow_half: true, ..RatingInput::default() };
        let star = star_attrs(&state(2.5), &input, 3);
        assert!(star.aria_checked);
        assert_eq!(star.aria_label, "2.5 stars");
        assert_eq!(star.tabindex, 0);
        assert_eq!(star_attrs(&state(2.5), &input, 1).aria_label, "1 star");
        assert_eq!(star_attrs(&state(2.5), &input, 1).tabindex, -1);
    }

    #[test]
    fn unrated_tabs_to_first_star_with_none_checked() {
        let input = RatingInput::default();
        let first = star_attrs(&state(0.0), &input, 1);
        assert!(!first.aria_checked);
        assert_eq!(first.tabindex, 0);
    }

    #[test]
    fn read_only_root_is_an_image() {
        let input = RatingInput { read_only: true, ..RatingInput::default() };
        let root = root_attrs(&state(3.5), &input, "Rating");
        assert_eq!((root.role, root.aria_label.as_str()), ("img", "Rated 3.5 out of 5"));
    }

    #[test]
    fn picks_and_keys() {
        assert!((pick(3, true) - 2.5).abs() < f64::EPSILON);
        assert!((pick(3, false) - 3.0).abs() < f64::EPSILON);
        assert_eq!(keydown("End"), Some(RatingEvent::Key(RatingKey::Last)));
        assert_eq!(keydown("a"), None);
    }
}
