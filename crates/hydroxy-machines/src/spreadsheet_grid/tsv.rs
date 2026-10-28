//! Tab-separated text, as spreadsheets copy and paste it.

/// Rows of fields from tab-separated text. Fields in double quotes may hold
/// tabs, newlines and doubled quotes, as spreadsheets write them. A final
/// newline does not add an empty row.
#[must_use]
pub fn parse_tsv(text: &str) -> Vec<Vec<String>> {
    let mut rows = Vec::new();
    let mut row = Vec::new();
    let mut field = String::new();
    let mut quoted = false;
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if quoted {
            if c != '"' {
                field.push(c);
            } else if chars.peek() == Some(&'"') {
                chars.next();
                field.push('"');
            } else {
                quoted = false;
            }
            continue;
        }
        match c {
            '"' if field.is_empty() => quoted = true,
            '\t' => row.push(std::mem::take(&mut field)),
            '\r' => {}
            '\n' => {
                row.push(std::mem::take(&mut field));
                rows.push(std::mem::take(&mut row));
            }
            _ => field.push(c),
        }
    }
    if !field.is_empty() || !row.is_empty() {
        row.push(field);
        rows.push(row);
    }
    rows
}

/// Tab-separated text from rows of fields, quoting fields that need it.
#[must_use]
pub fn to_tsv(rows: &[Vec<String>]) -> String {
    let quote = |f: &String| {
        if f.contains(['\t', '\n', '"']) { format!("\"{}\"", f.replace('"', "\"\"")) } else { f.clone() }
    };
    rows.iter().map(|r| r.iter().map(quote).collect::<Vec<_>>().join("\t")).collect::<Vec<_>>().join("\n")
}
