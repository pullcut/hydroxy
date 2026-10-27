//! CSV export of a chart's data table (RFC 4180): fields holding a comma, quote or line
//! break are quoted, quotes are doubled, and rows end in CRLF.

/// One CSV field, quoted when it needs to be.
#[must_use]
pub fn field(text: &str) -> String {
    if text.contains([',', '"', '\n', '\r']) { format!("\"{}\"", text.replace('"', "\"\"")) } else { text.to_string() }
}

/// The rows as CSV text, each row's cells joined by commas.
#[must_use]
pub fn to_csv(rows: &[Vec<String>]) -> String {
    rows.iter().map(|r| r.iter().map(|c| field(c.trim())).collect::<Vec<_>>().join(",") + "\r\n").collect()
}

/// A safe file name for `name`: letters, digits and dashes, ending in `.csv`.
#[must_use]
pub fn file_name(name: &str) -> String {
    let slug: String = name.chars().map(|c| if c.is_alphanumeric() { c.to_ascii_lowercase() } else { '-' }).collect();
    let slug = slug.split('-').filter(|s| !s.is_empty()).collect::<Vec<_>>().join("-");
    format!("{}.csv", if slug.is_empty() { "chart" } else { &slug })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quotes_only_what_needs_it() {
        let rows = vec![
            vec![String::new(), "Revenue".into(), "Note".into()],
            vec!["Jan".into(), "£1,200".into(), "said \"hi\"".into()],
            vec!["Feb".into(), " 900 ".into(), "two\nlines".into()],
        ];
        assert_eq!(to_csv(&rows), ",Revenue,Note\r\nJan,\"£1,200\",\"said \"\"hi\"\"\"\r\nFeb,900,\"two\nlines\"\r\n");
    }

    #[test]
    fn file_names_are_slugs() {
        assert_eq!(file_name("Weekly active users"), "weekly-active-users.csv");
        assert_eq!(file_name("  !! "), "chart.csv");
    }
}
