//! Presence bar: who is here right now. Orders people (active, then idle, then
//! away), splits them into the visible avatar stack and an overflow, and words
//! the summaries screen readers hear. No state: the host owns the room.

/// How present someone is.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PresenceStatus {
    /// Here and doing things.
    #[default]
    Active,
    /// Here, but hasn't done anything for a while.
    Idle,
    /// Tab hidden or stepped away.
    Away,
}

impl PresenceStatus {
    /// The `data-status` value, also the spoken word.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            PresenceStatus::Active => "active",
            PresenceStatus::Idle => "idle",
            PresenceStatus::Away => "away",
        }
    }
}

/// Display order: active first, then idle, then away. Stable, so the host's order
/// (say, join time) holds within each status.
#[must_use]
pub fn order(statuses: &[PresenceStatus]) -> Vec<usize> {
    let mut idx: Vec<usize> = (0..statuses.len()).collect();
    idx.sort_by_key(|&i| statuses[i]);
    idx
}

/// How many avatars to show for `len` people when at most `max` slots fit.
/// A "+1" would take a slot to hide one person, so in that case show them all.
#[must_use]
pub fn visible_count(len: usize, max: usize) -> usize {
    if len <= max { len } else { max.saturating_sub(1).max(1) }
}

/// One or two letters from a name: "Ana Lima" → "AL", "ana" → "A".
#[must_use]
pub fn initials(name: &str) -> String {
    let mut words = name.split_whitespace().filter_map(|w| w.chars().next());
    let first = words.next();
    let last = words.next_back();
    first.into_iter().chain(last).flat_map(char::to_uppercase).collect()
}

/// A stable colour slot (0..6) for someone without one, from their id.
#[must_use]
pub fn colour_slot(id: &str) -> usize {
    // FNV-1a: tiny, stable across runs and platforms.
    let hash = id.bytes().fold(0x811c_9dc5_u32, |h, b| (h ^ u32::from(b)).wrapping_mul(0x0100_0193));
    hash as usize % 6
}

/// The group's accessible name: "3 people here: Ana Lima, Ben Ode and Cy Ray".
/// Lists at most four names, then "and N others".
#[must_use]
pub fn summary(names: &[&str]) -> String {
    let n = names.len();
    let head = match n {
        0 => return "Nobody else here".to_string(),
        1 => "1 person here".to_string(),
        _ => format!("{n} people here"),
    };
    format!("{head}: {}", list(names, 4))
}

/// Join names as prose, naming at most `max` and counting the rest:
/// "Ana", "Ana and Ben", "Ana, Ben and Cy", "Ana, Ben and 3 others".
#[must_use]
pub fn list(names: &[&str], max: usize) -> String {
    let max = max.max(1);
    if names.len() <= max {
        return match names {
            [] => String::new(),
            [one] => (*one).to_string(),
            [rest @ .., last] => format!("{} and {last}", rest.join(", ")),
        };
    }
    let shown = max.saturating_sub(1).max(1);
    let others = names.len() - shown;
    let noun = if others == 1 { "other" } else { "others" };
    format!("{} and {others} {noun}", names[..shown].join(", "))
}

/// The overflow button's name: "Show 3 more people".
#[must_use]
pub fn overflow_label(hidden: usize) -> String {
    if hidden == 1 { "Show 1 more person".to_string() } else { format!("Show {hidden} more people") }
}

/// One person's accessible name: "Ana Lima (you), idle".
#[must_use]
pub fn person_label(name: &str, status: PresenceStatus, is_self: bool) -> String {
    let you = if is_self { " (you)" } else { "" };
    format!("{name}{you}, {}", status.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;
    use PresenceStatus::{Active, Away, Idle};

    #[test]
    fn active_first_and_stable_within_status() {
        assert_eq!(order(&[Away, Active, Idle, Active]), vec![1, 3, 2, 0]);
    }

    #[test]
    fn never_spends_a_slot_on_plus_one() {
        assert_eq!(visible_count(4, 4), 4);
        assert_eq!(visible_count(5, 4), 3);
        assert_eq!(visible_count(9, 1), 1);
        assert_eq!(visible_count(0, 4), 0);
    }

    #[test]
    fn initials_take_first_and_last_word() {
        assert_eq!(initials("Ana Maria Lima"), "AL");
        assert_eq!(initials("  ben "), "B");
        assert_eq!(initials(""), "");
    }

    #[test]
    fn colour_slot_is_stable_and_in_range() {
        assert_eq!(colour_slot("u-1"), colour_slot("u-1"));
        assert!((0..50).all(|i| colour_slot(&format!("user-{i}")) < 6));
    }

    #[test]
    fn summaries_read_as_prose() {
        assert_eq!(summary(&[]), "Nobody else here");
        assert_eq!(summary(&["Ana"]), "1 person here: Ana");
        assert_eq!(summary(&["Ana", "Ben", "Cy"]), "3 people here: Ana, Ben and Cy");
        assert_eq!(summary(&["A", "B", "C", "D", "E", "F"]), "6 people here: A, B, C and 3 others");
        assert_eq!(list(&["A", "B", "C"], 2), "A and 2 others");
        assert_eq!(overflow_label(1), "Show 1 more person");
        assert_eq!(person_label("Ana", Idle, true), "Ana (you), idle");
    }
}
