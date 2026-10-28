//! The JSON value type, its parser with line:column errors, and the serializer.

use std::fmt::Write as _;

/// Deepest nesting the parser accepts, so hostile input can't overflow the stack.
pub const MAX_DEPTH: usize = 512;

/// A parsed JSON value. Object keys keep their source order.
#[derive(Debug, Clone, PartialEq)]
pub enum JsonValue {
    /// `null`.
    Null,
    /// `true` or `false`.
    Bool(bool),
    /// A number, kept as its source text so nothing is lost to float rounding.
    Number(String),
    /// A string, unescaped.
    String(String),
    /// An array.
    Array(Vec<JsonValue>),
    /// An object, in source order.
    Object(Vec<(String, JsonValue)>),
}

impl JsonValue {
    /// Number of children: items or entries. Zero for primitives.
    #[must_use]
    pub fn len(&self) -> usize {
        match self {
            JsonValue::Array(a) => a.len(),
            JsonValue::Object(o) => o.len(),
            _ => 0,
        }
    }

    /// Whether this has no children (always true for primitives).
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Where and why parsing failed. `line` and `column` count from 1, columns in chars.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    /// What went wrong.
    pub message: String,
    /// Line of the problem.
    pub line: usize,
    /// Column of the problem.
    pub column: usize,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} at line {}, column {}", self.message, self.line, self.column)
    }
}

impl std::error::Error for ParseError {}

struct Parser<'a> {
    src: &'a str,
    pos: usize,
    depth: usize,
}

impl Parser<'_> {
    fn error(&self, message: impl Into<String>) -> ParseError {
        let before = &self.src[..self.pos.min(self.src.len())];
        let line = before.matches('\n').count() + 1;
        let column = before.rsplit('\n').next().unwrap_or("").chars().count() + 1;
        ParseError { message: message.into(), line, column }
    }

    fn peek(&self) -> Option<u8> {
        self.src.as_bytes().get(self.pos).copied()
    }

    fn skip_ws(&mut self) {
        while matches!(self.peek(), Some(b' ' | b'\t' | b'\n' | b'\r')) {
            self.pos += 1;
        }
    }

    fn expect(&mut self, b: u8) -> Result<(), ParseError> {
        if self.peek() == Some(b) {
            self.pos += 1;
            Ok(())
        } else {
            Err(self.unexpected(&format!("'{}'", b as char)))
        }
    }

    fn unexpected(&self, wanted: &str) -> ParseError {
        match self.src[self.pos.min(self.src.len())..].chars().next() {
            None => self.error(format!("Unexpected end of input, expected {wanted}")),
            Some(c) => self.error(format!("Unexpected '{c}', expected {wanted}")),
        }
    }

    fn value(&mut self) -> Result<JsonValue, ParseError> {
        self.skip_ws();
        match self.peek() {
            Some(b'{') => self.nested(Self::object),
            Some(b'[') => self.nested(Self::array),
            Some(b'"') => self.string().map(JsonValue::String),
            Some(b't') => self.word("true", JsonValue::Bool(true)),
            Some(b'f') => self.word("false", JsonValue::Bool(false)),
            Some(b'n') => self.word("null", JsonValue::Null),
            Some(b'-' | b'0'..=b'9') => self.number(),
            _ => Err(self.unexpected("a value")),
        }
    }

    fn nested(&mut self, f: fn(&mut Self) -> Result<JsonValue, ParseError>) -> Result<JsonValue, ParseError> {
        self.depth += 1;
        if self.depth > MAX_DEPTH {
            return Err(self.error("Nested too deeply"));
        }
        let v = f(self);
        self.depth -= 1;
        v
    }

    fn word(&mut self, word: &str, v: JsonValue) -> Result<JsonValue, ParseError> {
        if self.src[self.pos..].starts_with(word) {
            self.pos += word.len();
            Ok(v)
        } else {
            Err(self.unexpected("a value"))
        }
    }

    fn digits(&mut self) -> usize {
        let start = self.pos;
        while self.peek().is_some_and(|b| b.is_ascii_digit()) {
            self.pos += 1;
        }
        self.pos - start
    }

    fn number(&mut self) -> Result<JsonValue, ParseError> {
        let start = self.pos;
        if self.peek() == Some(b'-') {
            self.pos += 1;
        }
        match self.peek() {
            Some(b'0') => self.pos += 1,
            Some(b'1'..=b'9') => {
                self.digits();
            }
            _ => return Err(self.unexpected("a digit")),
        }
        if self.peek() == Some(b'.') {
            self.pos += 1;
            if self.digits() == 0 {
                return Err(self.unexpected("a digit"));
            }
        }
        if matches!(self.peek(), Some(b'e' | b'E')) {
            self.pos += 1;
            if matches!(self.peek(), Some(b'+' | b'-')) {
                self.pos += 1;
            }
            if self.digits() == 0 {
                return Err(self.unexpected("a digit"));
            }
        }
        Ok(JsonValue::Number(self.src[start..self.pos].to_string()))
    }

    fn hex4(&mut self) -> Result<u32, ParseError> {
        let hex = self.src.get(self.pos..self.pos + 4).filter(|h| h.bytes().all(|b| b.is_ascii_hexdigit()));
        let Some(hex) = hex else { return Err(self.error("Invalid \\u escape")) };
        self.pos += 4;
        Ok(u32::from_str_radix(hex, 16).unwrap_or(0))
    }

    fn string(&mut self) -> Result<String, ParseError> {
        self.expect(b'"')?;
        let mut out = String::new();
        loop {
            let rest = &self.src[self.pos..];
            // Copy the run up to the next quote, backslash or control char in one go.
            let run = rest.find(|c: char| c == '"' || c == '\\' || c < ' ').unwrap_or(rest.len());
            out.push_str(&rest[..run]);
            self.pos += run;
            match self.peek() {
                None => return Err(self.error("Unterminated string")),
                Some(b'"') => {
                    self.pos += 1;
                    return Ok(out);
                }
                Some(b'\\') => {
                    self.pos += 1;
                    let Some(e) = self.peek() else { return Err(self.error("Unterminated string")) };
                    self.pos += 1;
                    match e {
                        b'"' => out.push('"'),
                        b'\\' => out.push('\\'),
                        b'/' => out.push('/'),
                        b'b' => out.push('\u{8}'),
                        b'f' => out.push('\u{c}'),
                        b'n' => out.push('\n'),
                        b'r' => out.push('\r'),
                        b't' => out.push('\t'),
                        b'u' => {
                            let mut code = self.hex4()?;
                            if (0xD800..0xDC00).contains(&code) && self.src[self.pos..].starts_with("\\u") {
                                let save = self.pos;
                                self.pos += 2;
                                let low = self.hex4()?;
                                if (0xDC00..0xE000).contains(&low) {
                                    code = 0x10000 + ((code - 0xD800) << 10) + (low - 0xDC00);
                                } else {
                                    self.pos = save;
                                }
                            }
                            out.push(char::from_u32(code).unwrap_or('\u{FFFD}'));
                        }
                        _ => {
                            self.pos -= 1;
                            return Err(self.error("Invalid escape"));
                        }
                    }
                }
                Some(_) => return Err(self.error("Control character in string")),
            }
        }
    }

    fn array(&mut self) -> Result<JsonValue, ParseError> {
        self.expect(b'[')?;
        let mut items = Vec::new();
        self.skip_ws();
        if self.peek() == Some(b']') {
            self.pos += 1;
            return Ok(JsonValue::Array(items));
        }
        loop {
            items.push(self.value()?);
            self.skip_ws();
            match self.peek() {
                Some(b',') => self.pos += 1,
                Some(b']') => {
                    self.pos += 1;
                    return Ok(JsonValue::Array(items));
                }
                _ => return Err(self.unexpected("',' or ']'")),
            }
        }
    }

    fn object(&mut self) -> Result<JsonValue, ParseError> {
        self.expect(b'{')?;
        let mut entries = Vec::new();
        self.skip_ws();
        if self.peek() == Some(b'}') {
            self.pos += 1;
            return Ok(JsonValue::Object(entries));
        }
        loop {
            self.skip_ws();
            if self.peek() != Some(b'"') {
                return Err(self.unexpected("a string key"));
            }
            let key = self.string()?;
            self.skip_ws();
            self.expect(b':')?;
            entries.push((key, self.value()?));
            self.skip_ws();
            match self.peek() {
                Some(b',') => self.pos += 1,
                Some(b'}') => {
                    self.pos += 1;
                    return Ok(JsonValue::Object(entries));
                }
                _ => return Err(self.unexpected("',' or '}'")),
            }
        }
    }
}

/// Parse JSON text (RFC 8259).
///
/// # Errors
/// A [`ParseError`] naming the first problem and where it is.
pub fn parse(src: &str) -> Result<JsonValue, ParseError> {
    let mut p = Parser { src, pos: 0, depth: 0 };
    let v = p.value()?;
    p.skip_ws();
    if p.pos < src.len() {
        return Err(p.unexpected("end of input"));
    }
    Ok(v)
}

/// `s` as a quoted JSON string.
#[must_use]
pub fn quote(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c < ' ' => {
                let _ = write!(out, "\\u{:04x}", u32::from(c));
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// Serialize `v`. `pretty` indents by two spaces per level.
#[must_use]
pub fn to_json(v: &JsonValue, pretty: bool) -> String {
    let mut out = String::new();
    write_json(&mut out, v, pretty, 0);
    out
}

fn write_json(out: &mut String, v: &JsonValue, pretty: bool, indent: usize) {
    let newline = |out: &mut String, n: usize| {
        if pretty {
            out.push('\n');
            out.push_str(&"  ".repeat(n));
        }
    };
    match v {
        JsonValue::Null => out.push_str("null"),
        JsonValue::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
        JsonValue::Number(n) => out.push_str(n),
        JsonValue::String(s) => out.push_str(&quote(s)),
        JsonValue::Array(items) if items.is_empty() => out.push_str("[]"),
        JsonValue::Object(entries) if entries.is_empty() => out.push_str("{}"),
        JsonValue::Array(items) => {
            out.push('[');
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                newline(out, indent + 1);
                write_json(out, item, pretty, indent + 1);
            }
            newline(out, indent);
            out.push(']');
        }
        JsonValue::Object(entries) => {
            out.push('{');
            for (i, (k, item)) in entries.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                newline(out, indent + 1);
                out.push_str(&quote(k));
                out.push_str(if pretty { ": " } else { ":" });
                write_json(out, item, pretty, indent + 1);
            }
            newline(out, indent);
            out.push('}');
        }
    }
}

/// The path of `key` under `parent`: `$.name` for identifiers, `$["a b"]` otherwise.
#[must_use]
pub fn key_path(parent: &str, key: &str) -> String {
    let mut chars = key.chars();
    let ident = chars.next().is_some_and(|c| c.is_ascii_alphabetic() || c == '_' || c == '$')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '$');
    if ident { format!("{parent}.{key}") } else { format!("{parent}[{}]", quote(key)) }
}
