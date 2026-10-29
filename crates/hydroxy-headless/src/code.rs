//! Light syntax colouring for code blocks: comments, strings, numbers, keywords and
//! capitalised type names. A tokenizer, not a parser; enough to make samples readable.

/// What a highlighted run is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Token {
    /// A comment.
    Comment,
    /// A string literal.
    Str,
    /// A number.
    Number,
    /// A language keyword.
    Keyword,
    /// A capitalised name, usually a type.
    Type,
}

impl Token {
    /// The `data-hl` value the styles key off.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Token::Comment => "c",
            Token::Str => "s",
            Token::Number => "n",
            Token::Keyword => "k",
            Token::Type => "t",
        }
    }
}

const JS_KEYWORDS: &[&str] = &[
    "const",
    "let",
    "var",
    "function",
    "return",
    "if",
    "else",
    "for",
    "while",
    "await",
    "async",
    "import",
    "from",
    "export",
    "default",
    "new",
    "class",
    "extends",
    "type",
    "interface",
    "implements",
    "true",
    "false",
    "null",
    "undefined",
];
const SQL_KEYWORDS: &[&str] = &[
    "select", "from", "where", "insert", "into", "values", "create", "table", "index", "unique", "on", "begin",
    "commit", "alter", "add", "and", "or", "not", "null", "join", "group", "by", "order", "limit",
];
const RUST_KEYWORDS: &[&str] = &[
    "as", "async", "await", "const", "crate", "else", "enum", "false", "fn", "for", "if", "impl", "in", "let", "loop",
    "match", "mod", "move", "mut", "pub", "ref", "return", "self", "Self", "static", "struct", "super", "trait",
    "true", "type", "use", "where", "while",
];

struct Lang {
    comment: Option<&'static str>,
    keywords: &'static [&'static str],
    ignore_case: bool,
}

fn lang(name: &str) -> Option<Lang> {
    let name = name.to_ascii_lowercase();
    Some(match name.as_str() {
        "rust" | "rs" => Lang { comment: Some("//"), keywords: RUST_KEYWORDS, ignore_case: false },
        "ts" | "tsx" | "js" | "javascript" | "typescript" | "json" => {
            Lang { comment: Some("//"), keywords: JS_KEYWORDS, ignore_case: false }
        }
        "bash" | "sh" | "yaml" | "yml" | "toml" => Lang { comment: Some("#"), keywords: &[], ignore_case: false },
        "sql" => Lang { comment: Some("--"), keywords: SQL_KEYWORDS, ignore_case: true },
        _ => return None,
    })
}

/// Split `code` into runs, each plain (`None`) or coloured. Unknown languages come
/// back as one plain run. Concatenating the runs gives back `code` exactly.
#[must_use]
pub fn highlight<'a>(code: &'a str, language: &str) -> Vec<(Option<Token>, &'a str)> {
    let Some(lang) = lang(language) else { return vec![(None, code)] };
    let bytes = code.as_bytes();
    let mut out: Vec<(Option<Token>, &str)> = Vec::new();
    let mut plain_start = 0;
    let mut i = 0;
    let push = |out: &mut Vec<_>, plain_start: &mut usize, start: usize, end: usize, tok: Token| {
        if *plain_start < start {
            out.push((None, &code[*plain_start..start]));
        }
        out.push((Some(tok), &code[start..end]));
        *plain_start = end;
    };
    while i < bytes.len() {
        let rest = &code[i..];
        let c = bytes[i];
        if lang.comment.is_some_and(|m| rest.starts_with(m)) {
            let end = rest.find('\n').map_or(code.len(), |n| i + n);
            push(&mut out, &mut plain_start, i, end, Token::Comment);
            i = end;
        } else if c == b'"' || c == b'\'' || c == b'`' {
            // Rust lifetimes and chars: only treat ' as a quote when it closes within a few bytes.
            let mut j = i + 1;
            while j < bytes.len() && bytes[j] != c && bytes[j] != b'\n' {
                j += if bytes[j] == b'\\' { 2 } else { 1 };
            }
            let closed = j < bytes.len() && bytes[j] == c;
            if c == b'\'' && (!closed || j - i > 4) {
                i += 1;
                continue;
            }
            let end = (j + 1).min(bytes.len());
            push(&mut out, &mut plain_start, i, end, Token::Str);
            i = end;
        } else if c.is_ascii_digit() && (i == 0 || !is_ident(bytes[i - 1])) {
            let mut j = i;
            while j < bytes.len() && (bytes[j].is_ascii_alphanumeric() || bytes[j] == b'.' || bytes[j] == b'_') {
                j += 1;
            }
            push(&mut out, &mut plain_start, i, j, Token::Number);
            i = j;
        } else if is_ident(c) && (i == 0 || !is_ident(bytes[i - 1])) {
            let mut j = i;
            while j < bytes.len() && is_ident(bytes[j]) {
                j += 1;
            }
            let word = &code[i..j];
            let keyword =
                lang.keywords.iter().any(|k| if lang.ignore_case { k.eq_ignore_ascii_case(word) } else { *k == word });
            if keyword {
                push(&mut out, &mut plain_start, i, j, Token::Keyword);
            } else if word.len() > 1 && word.as_bytes()[0].is_ascii_uppercase() {
                push(&mut out, &mut plain_start, i, j, Token::Type);
            }
            i = j;
        } else {
            i += 1;
        }
    }
    if plain_start < code.len() {
        out.push((None, &code[plain_start..]));
    }
    out
}

const fn is_ident(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(code: &str, lang: &str) -> Vec<(Option<&'static str>, String)> {
        highlight(code, lang).into_iter().map(|(t, s)| (t.map(Token::as_str), s.to_string())).collect()
    }

    #[test]
    fn runs_rebuild_the_source() {
        let src = "let x = \"hi\"; // note\nfn main() -> Option<u8> { 42 }";
        assert_eq!(highlight(src, "rust").into_iter().map(|(_, s)| s).collect::<String>(), src);
    }

    #[test]
    fn colours_rust() {
        let k = kinds("pub fn go(n: u8) -> Option<u8> { 42 } // done", "rust");
        assert!(k.contains(&(Some("k"), "pub".into())));
        assert!(k.contains(&(Some("k"), "fn".into())));
        assert!(k.contains(&(Some("t"), "Option".into())));
        assert!(k.contains(&(Some("n"), "42".into())));
        assert!(k.contains(&(Some("c"), "// done".into())));
    }

    #[test]
    fn lifetimes_are_not_strings_and_sql_ignores_case() {
        assert!(!kinds("fn f<'a>(x: &'a str) {}", "rust").iter().any(|(t, _)| *t == Some("s")));
        assert!(kinds("SELECT id FROM t", "sql").contains(&(Some("k"), "SELECT".into())));
    }

    #[test]
    fn unknown_languages_stay_plain() {
        assert_eq!(kinds("anything", "text"), vec![(None, "anything".to_string())]);
    }
}
