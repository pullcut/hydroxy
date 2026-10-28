//! Caret bookkeeping for fields that reformat as you type (money, phone). The
//! field keeps "significant" characters (digits, a sign, a decimal mark) and
//! re-inserts literals (group separators, spaces, brackets) around them. The
//! caret is carried across a reformat by counting significant characters.

/// How many significant characters sit before char index `caret`.
pub(crate) fn sig_before(text: &str, caret: usize, sig: impl Fn(char) -> bool) -> usize {
    text.chars().take(caret).filter(|c| sig(*c)).count()
}

/// The char index just after the `n`th significant character, `0` for none,
/// the end when there are fewer than `n`.
pub(crate) fn caret_after(text: &str, n: usize, sig: impl Fn(char) -> bool) -> usize {
    if n == 0 {
        return 0;
    }
    let mut seen = 0;
    for (i, c) in text.chars().enumerate() {
        if sig(c) {
            seen += 1;
            if seen == n {
                return i + 1;
            }
        }
    }
    text.chars().count()
}

/// Whether going from `prev` to `text` only deleted one literal, such as
/// Backspace over a group separator. Reformatting would put it straight back,
/// so the caller deletes the significant character before it instead.
pub(crate) fn deleted_literal(prev: &str, text: &str, sig: impl Fn(char) -> bool + Copy) -> bool {
    prev.chars().count() == text.chars().count() + 1
        && prev.chars().filter(|c| sig(*c)).eq(text.chars().filter(|c| sig(*c)))
}

/// Remove the last significant character before `caret`. Returns the new text
/// and caret.
pub(crate) fn drop_sig_before(text: &str, caret: usize, sig: impl Fn(char) -> bool) -> (String, usize) {
    let chars: Vec<char> = text.chars().collect();
    let caret = caret.min(chars.len());
    match (0..caret).rev().find(|&i| sig(chars[i])) {
        Some(at) => {
            let mut out = chars;
            out.remove(at);
            (out.into_iter().collect(), at)
        }
        None => (text.to_string(), caret),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn digit(c: char) -> bool {
        c.is_ascii_digit()
    }

    #[test]
    fn caret_survives_regrouping() {
        // "1,234" with the caret after "12": two digits before it.
        assert_eq!(sig_before("1,234", 3, digit), 2);
        assert_eq!(caret_after("12,345", 2, digit), 2);
        assert_eq!(caret_after("(201) 555", 3, digit), 4);
        assert_eq!(caret_after("12", 9, digit), 2);
        assert_eq!(caret_after("(2", 0, digit), 0);
    }

    #[test]
    fn backspace_over_a_separator_takes_the_digit_before() {
        assert!(deleted_literal("1,234", "1234", digit));
        assert!(!deleted_literal("1,234", "1,23", digit));
        assert_eq!(drop_sig_before("1234", 1, digit), ("234".to_string(), 0));
    }
}
