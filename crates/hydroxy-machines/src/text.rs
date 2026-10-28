//! Small text helpers shared by machines.

/// `"1 item"` or `"3 items"`: `n` and `word`, with an `s` unless `n` is 1.
#[must_use]
pub fn plural(n: usize, word: &str) -> String {
    if n == 1 { format!("1 {word}") } else { format!("{n} {word}s") }
}

#[cfg(test)]
mod tests {
    use super::plural;

    #[test]
    fn pluralises_unless_one() {
        assert_eq!(plural(1, "shape"), "1 shape");
        assert_eq!(plural(0, "shape"), "0 shapes");
        assert_eq!(plural(3, "shape"), "3 shapes");
    }
}
