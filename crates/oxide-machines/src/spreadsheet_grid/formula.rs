//! Spreadsheet formulas. A cell whose text starts with `=` is a formula:
//! numbers, A1 references, ranges (`A1:B3`), `+ - * /` with the usual
//! precedence, parentheses, unary minus and `SUM`, `AVERAGE`, `MIN`, `MAX`.
//! [`recalc`] evaluates every formula in dependency order and marks cycles.

use std::collections::{BTreeMap, HashMap, VecDeque};

/// A cell position as (row, column), both 0-based.
pub type Cell = (usize, usize);

/// Why a cell has no value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CellError {
    /// A reference outside the sheet, or a malformed one like `A0`.
    Ref,
    /// Division by zero, or the average of nothing.
    Div0,
    /// The cell depends on itself, directly or through other cells.
    Cycle,
    /// Text where a number was needed, a bare range, or a result too large.
    Value,
    /// An unknown function or name.
    Name,
    /// The formula does not parse.
    Syntax,
}

impl CellError {
    /// The text a spreadsheet shows for the error.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ref => "#REF!",
            Self::Div0 => "#DIV/0!",
            Self::Cycle => "#CYCLE!",
            Self::Value => "#VALUE!",
            Self::Name => "#NAME?",
            Self::Syntax => "#ERROR!",
        }
    }
}

/// What a cell evaluates to.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// Nothing in the cell. Counts as 0 in arithmetic.
    Empty,
    /// A number.
    Number(f64),
    /// Text that is not a number.
    Text(String),
    /// An error, shown as `#REF!` and friends.
    Error(CellError),
}

impl Value {
    /// The text shown in the cell.
    #[must_use]
    pub fn display(&self) -> String {
        match self {
            Self::Empty => String::new(),
            Self::Number(n) => format_number(*n),
            Self::Text(t) => t.clone(),
            Self::Error(e) => e.as_str().to_string(),
        }
    }

    /// `empty`, `number`, `text` or `error`, for styling.
    #[must_use]
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Empty => "empty",
            Self::Number(_) => "number",
            Self::Text(_) => "text",
            Self::Error(_) => "error",
        }
    }
}

/// A number as a sheet shows it: at most 10 decimals, no trailing zeros.
#[must_use]
pub fn format_number(n: f64) -> String {
    let s = format!("{n:.10}");
    let s = if s.contains('.') { s.trim_end_matches('0').trim_end_matches('.') } else { s.as_str() };
    if s == "-0" { "0".to_string() } else { s.to_string() }
}

/// The value of a plain (non-formula) cell: a number if it reads as one, else text.
#[must_use]
pub fn literal(raw: &str) -> Value {
    let t = raw.trim();
    if t.is_empty() {
        return Value::Empty;
    }
    match t.parse::<f64>() {
        Ok(n) if n.is_finite() => Value::Number(n),
        _ => Value::Text(raw.to_string()),
    }
}

/// Column letters for a 0-based index: 0 is `A`, 25 is `Z`, 26 is `AA`.
#[must_use]
pub fn column_name(col: usize) -> String {
    const LETTERS: &[u8; 26] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    let mut out = Vec::new();
    let mut n = col;
    loop {
        out.push(char::from(LETTERS[n % 26]));
        if n < 26 {
            break;
        }
        n = n / 26 - 1;
    }
    out.iter().rev().collect()
}

/// A cell's A1 name.
#[must_use]
pub fn cell_name((row, col): Cell) -> String {
    format!("{}{}", column_name(col), row + 1)
}

/// Parse an A1 name (case-insensitive, `$` ignored) into a cell. `None` for
/// anything else, including row 0.
#[must_use]
pub fn parse_cell(name: &str) -> Option<Cell> {
    let name: String = name.chars().filter(|c| *c != '$').collect();
    let split = name.find(|c: char| c.is_ascii_digit())?;
    let (letters, digits) = name.split_at(split);
    if letters.is_empty() || !letters.chars().all(|c| c.is_ascii_alphabetic()) || !digits.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    let mut col: usize = 0;
    for b in letters.bytes() {
        let digit = usize::from(b.to_ascii_uppercase() - b'A') + 1;
        col = col.checked_mul(26)?.checked_add(digit)?;
    }
    let row: usize = digits.parse().ok()?;
    (row >= 1).then(|| (row - 1, col - 1))
}

/// A binary operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Op {
    /// `+`
    Add,
    /// `-`
    Sub,
    /// `*`
    Mul,
    /// `/`
    Div,
}

/// A built-in function.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Func {
    /// Sum of the numbers.
    Sum,
    /// Mean of the numbers; `#DIV/0!` when there are none.
    Average,
    /// Smallest number; 0 when there are none.
    Min,
    /// Largest number; 0 when there are none.
    Max,
}

/// A parsed formula.
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// A number literal.
    Number(f64),
    /// A single cell.
    Ref(Cell),
    /// A rectangle, normalised so the first corner is the top left.
    Range(Cell, Cell),
    /// Unary minus.
    Neg(Box<Expr>),
    /// `left op right`.
    Binary(Op, Box<Expr>, Box<Expr>),
    /// A function call.
    Call(Func, Vec<Expr>),
}

impl Expr {
    /// Every rectangle the formula reads (a single cell is a 1×1 rectangle).
    pub fn references(&self, out: &mut Vec<(Cell, Cell)>) {
        match self {
            Self::Number(_) => {}
            Self::Ref(c) => out.push((*c, *c)),
            Self::Range(a, b) => out.push((*a, *b)),
            Self::Neg(e) => e.references(out),
            Self::Binary(_, l, r) => {
                l.references(out);
                r.references(out);
            }
            Self::Call(_, args) => args.iter().for_each(|a| a.references(out)),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum Tok {
    Num(f64),
    Word(String),
    Op(char),
    Open,
    Close,
    Comma,
    Colon,
}

fn tokenize(src: &str) -> Result<Vec<Tok>, CellError> {
    let chars: Vec<char> = src.chars().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c.is_whitespace() {
            i += 1;
        } else if c.is_ascii_digit() || c == '.' {
            let start = i;
            while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
                i += 1;
            }
            let text: String = chars[start..i].iter().collect();
            out.push(Tok::Num(text.parse().map_err(|_| CellError::Syntax)?));
        } else if c.is_ascii_alphabetic() || c == '$' {
            let start = i;
            while i < chars.len() && (chars[i].is_ascii_alphanumeric() || chars[i] == '$') {
                i += 1;
            }
            out.push(Tok::Word(chars[start..i].iter().filter(|c| **c != '$').collect::<String>().to_ascii_uppercase()));
        } else {
            out.push(match c {
                '+' | '-' | '*' | '/' => Tok::Op(c),
                '(' => Tok::Open,
                ')' => Tok::Close,
                ',' | ';' => Tok::Comma,
                ':' => Tok::Colon,
                _ => return Err(CellError::Syntax),
            });
            i += 1;
        }
    }
    Ok(out)
}

struct Parser {
    toks: Vec<Tok>,
    pos: usize,
}

impl Parser {
    fn peek(&self) -> Option<&Tok> {
        self.toks.get(self.pos)
    }

    fn next(&mut self) -> Option<Tok> {
        let t = self.toks.get(self.pos).cloned();
        self.pos += 1;
        t
    }

    fn expr(&mut self) -> Result<Expr, CellError> {
        let mut left = self.term()?;
        while let Some(Tok::Op(c @ ('+' | '-'))) = self.peek() {
            let op = if *c == '+' { Op::Add } else { Op::Sub };
            self.pos += 1;
            left = Expr::Binary(op, Box::new(left), Box::new(self.term()?));
        }
        Ok(left)
    }

    fn term(&mut self) -> Result<Expr, CellError> {
        let mut left = self.unary()?;
        while let Some(Tok::Op(c @ ('*' | '/'))) = self.peek() {
            let op = if *c == '*' { Op::Mul } else { Op::Div };
            self.pos += 1;
            left = Expr::Binary(op, Box::new(left), Box::new(self.unary()?));
        }
        Ok(left)
    }

    fn unary(&mut self) -> Result<Expr, CellError> {
        match self.peek() {
            Some(Tok::Op('-')) => {
                self.pos += 1;
                Ok(Expr::Neg(Box::new(self.unary()?)))
            }
            Some(Tok::Op('+')) => {
                self.pos += 1;
                self.unary()
            }
            _ => self.primary(),
        }
    }

    fn primary(&mut self) -> Result<Expr, CellError> {
        match self.next() {
            Some(Tok::Num(n)) => Ok(Expr::Number(n)),
            Some(Tok::Open) => {
                let e = self.expr()?;
                if self.next() != Some(Tok::Close) {
                    return Err(CellError::Syntax);
                }
                Ok(e)
            }
            Some(Tok::Word(word)) if self.peek() == Some(&Tok::Open) => {
                self.pos += 1;
                let func = match word.as_str() {
                    "SUM" => Func::Sum,
                    "AVERAGE" | "AVG" => Func::Average,
                    "MIN" => Func::Min,
                    "MAX" => Func::Max,
                    _ => return Err(CellError::Name),
                };
                let mut args = Vec::new();
                if self.peek() == Some(&Tok::Close) {
                    self.pos += 1;
                    return Ok(Expr::Call(func, args));
                }
                loop {
                    args.push(self.expr()?);
                    match self.next() {
                        Some(Tok::Comma) => {}
                        Some(Tok::Close) => return Ok(Expr::Call(func, args)),
                        _ => return Err(CellError::Syntax),
                    }
                }
            }
            Some(Tok::Word(word)) => {
                let a = reference(&word)?;
                if self.peek() != Some(&Tok::Colon) {
                    return Ok(Expr::Ref(a));
                }
                self.pos += 1;
                let Some(Tok::Word(end)) = self.next() else { return Err(CellError::Syntax) };
                let b = reference(&end)?;
                Ok(Expr::Range((a.0.min(b.0), a.1.min(b.1)), (a.0.max(b.0), a.1.max(b.1))))
            }
            _ => Err(CellError::Syntax),
        }
    }
}

/// A word that should be a cell: `#REF!` if it looks like one but is not
/// (`A0`, absurd columns), `#NAME?` if it is just a name.
fn reference(word: &str) -> Result<Cell, CellError> {
    parse_cell(word).ok_or(if word.chars().any(|c| c.is_ascii_digit()) { CellError::Ref } else { CellError::Name })
}

/// Parse a formula's text after the leading `=`.
///
/// # Errors
/// The error the cell shows: `#ERROR!` for bad syntax, `#NAME?` for unknown
/// functions, `#REF!` for malformed references.
pub fn parse(src: &str) -> Result<Expr, CellError> {
    let mut p = Parser { toks: tokenize(src)?, pos: 0 };
    let e = p.expr()?;
    if p.pos != p.toks.len() {
        return Err(CellError::Syntax);
    }
    Ok(e)
}

fn number(v: Value) -> Result<f64, CellError> {
    match v {
        Value::Empty => Ok(0.0),
        Value::Number(n) => Ok(n),
        Value::Text(t) => t.trim().parse::<f64>().ok().filter(|n| n.is_finite()).ok_or(CellError::Value),
        Value::Error(e) => Err(e),
    }
}

fn in_sheet((row, col): Cell, size: Cell) -> Result<Cell, CellError> {
    if row < size.0 && col < size.1 { Ok((row, col)) } else { Err(CellError::Ref) }
}

fn scalar(expr: &Expr, get: &dyn Fn(Cell) -> Value, size: Cell) -> Result<Value, CellError> {
    match expr {
        Expr::Number(n) => Ok(Value::Number(*n)),
        Expr::Ref(c) => match get(in_sheet(*c, size)?) {
            Value::Error(e) => Err(e),
            v => Ok(v),
        },
        Expr::Range(..) => Err(CellError::Value),
        Expr::Neg(e) => Ok(Value::Number(-number(scalar(e, get, size)?)?)),
        Expr::Binary(op, l, r) => {
            let (lhs, rhs) = (number(scalar(l, get, size)?)?, number(scalar(r, get, size)?)?);
            let n = match op {
                Op::Add => lhs + rhs,
                Op::Sub => lhs - rhs,
                Op::Mul => lhs * rhs,
                Op::Div if matches!(rhs.classify(), std::num::FpCategory::Zero) => return Err(CellError::Div0),
                Op::Div => lhs / rhs,
            };
            if n.is_finite() { Ok(Value::Number(n)) } else { Err(CellError::Value) }
        }
        Expr::Call(func, args) => {
            let mut nums = Vec::new();
            for arg in args {
                match arg {
                    // Cells passed to a function skip text and blanks, like a spreadsheet.
                    Expr::Ref(from) | Expr::Range(from, _) => {
                        let to = if let Expr::Range(_, to) = arg { *to } else { *from };
                        in_sheet(to, size)?;
                        for row in from.0..=to.0 {
                            for col in from.1..=to.1 {
                                match get((row, col)) {
                                    Value::Number(n) => nums.push(n),
                                    Value::Error(e) => return Err(e),
                                    Value::Empty | Value::Text(_) => {}
                                }
                            }
                        }
                    }
                    e => nums.push(number(scalar(e, get, size)?)?),
                }
            }
            let n = match func {
                Func::Sum => nums.iter().sum(),
                Func::Average if nums.is_empty() => return Err(CellError::Div0),
                Func::Average => mean(&nums),
                Func::Min => nums.iter().copied().reduce(f64::min).unwrap_or(0.0),
                Func::Max => nums.iter().copied().reduce(f64::max).unwrap_or(0.0),
            };
            if n.is_finite() { Ok(Value::Number(n)) } else { Err(CellError::Value) }
        }
    }
}

#[allow(clippy::cast_precision_loss, reason = "argument counts are far below 2^52")]
fn mean(nums: &[f64]) -> f64 {
    nums.iter().sum::<f64>() / nums.len() as f64
}

/// Evaluate a parsed formula. `get` returns another cell's value; `size` is
/// the sheet's (rows, columns), and references outside it are `#REF!`. A
/// formula that is just a blank reference evaluates to 0.
pub fn eval(expr: &Expr, get: &dyn Fn(Cell) -> Value, size: Cell) -> Value {
    match scalar(expr, get, size) {
        Ok(Value::Empty) => Value::Number(0.0),
        Ok(v) => v,
        Err(e) => Value::Error(e),
    }
}

/// Evaluate every formula in `cells` (raw text by cell) in dependency order.
/// Returns the value of each formula cell; plain cells are their [`literal`].
/// Formulas in a cycle, and formulas that depend on one, are `#CYCLE!`.
///
/// Iterative (Kahn's algorithm), so long reference chains cannot overflow the stack.
#[must_use]
pub fn recalc(cells: &BTreeMap<Cell, String>, size: Cell) -> HashMap<Cell, Value> {
    let parsed: HashMap<Cell, Result<Expr, CellError>> =
        cells.iter().filter_map(|(c, raw)| raw.strip_prefix('=').map(|f| (*c, parse(f)))).collect();
    let mut waiting: HashMap<Cell, usize> = HashMap::new();
    let mut dependents: HashMap<Cell, Vec<Cell>> = HashMap::new();
    let mut ready = VecDeque::new();
    for (cell, expr) in &parsed {
        let mut deps = Vec::new();
        if let Ok(expr) = expr {
            let mut rects = Vec::new();
            expr.references(&mut rects);
            for (a, b) in rects {
                let area = (b.0 - a.0 + 1).saturating_mul(b.1 - a.1 + 1);
                let inside = |c: &Cell| (a.0..=b.0).contains(&c.0) && (a.1..=b.1).contains(&c.1);
                // Walk whichever is smaller: the rectangle or the formula cells.
                if area <= parsed.len() {
                    for row in a.0..=b.0 {
                        deps.extend((a.1..=b.1).map(|col| (row, col)).filter(|c| parsed.contains_key(c)));
                    }
                } else {
                    deps.extend(parsed.keys().copied().filter(|c| inside(c)));
                }
            }
            deps.sort_unstable();
            deps.dedup();
        }
        if deps.is_empty() {
            ready.push_back(*cell);
        }
        waiting.insert(*cell, deps.len());
        for d in deps {
            dependents.entry(d).or_default().push(*cell);
        }
    }

    let mut values: HashMap<Cell, Value> = HashMap::with_capacity(parsed.len());
    while let Some(cell) = ready.pop_front() {
        let value = match &parsed[&cell] {
            Err(e) => Value::Error(*e),
            Ok(expr) => {
                let get = |c: Cell| values.get(&c).cloned().unwrap_or_else(|| cells.get(&c).map_or(Value::Empty, |r| literal(r)));
                eval(expr, &get, size)
            }
        };
        values.insert(cell, value);
        for d in dependents.get(&cell).into_iter().flatten() {
            if let Some(n) = waiting.get_mut(d) {
                *n -= 1;
                if *n == 0 {
                    ready.push_back(*d);
                }
            }
        }
    }
    for cell in parsed.keys() {
        values.entry(*cell).or_insert(Value::Error(CellError::Cycle));
    }
    values
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sheet(cells: &[(&str, &str)]) -> HashMap<Cell, Value> {
        let map = cells.iter().map(|(name, raw)| (parse_cell(name).unwrap(), (*raw).to_string())).collect();
        recalc(&map, (100, 26))
    }

    fn calc(formula: &str) -> Value {
        sheet(&[("Z100", formula)]).remove(&(99, 25)).unwrap()
    }

    fn at(values: &HashMap<Cell, Value>, name: &str) -> String {
        values[&parse_cell(name).unwrap()].display()
    }

    #[test]
    fn column_names_roll_over_like_a_spreadsheet() {
        let names: Vec<String> = [0, 1, 25, 26, 27, 51, 52, 701, 702].into_iter().map(column_name).collect();
        assert_eq!(names, ["A", "B", "Z", "AA", "AB", "AZ", "BA", "ZZ", "AAA"]);
        for col in 0..2000 {
            assert_eq!(parse_cell(&format!("{}7", column_name(col))), Some((6, col)));
        }
        assert_eq!(parse_cell("b3"), Some((2, 1)));
        assert_eq!(parse_cell("$C$10"), Some((9, 2)));
        assert_eq!(cell_name((9, 27)), "AB10");
        for bad in ["A0", "A", "3", "A1B", "", "ZZZZZZZZZZZZZZZZZZZZ1"] {
            assert_eq!(parse_cell(bad), None, "{bad}");
        }
    }

    #[test]
    fn arithmetic_follows_precedence_and_parentheses() {
        let cases = [
            ("=1+2*3", "7"),
            ("=(1+2)*3", "9"),
            ("=10-4-3", "3"),
            ("=8/4/2", "1"),
            ("=2*-3", "-6"),
            ("=-(2+3)*2", "-10"),
            ("=2--1", "3"),
            ("=+5", "5"),
            ("= 1.5 * 4 ", "6"),
            ("=.5+.25", "0.75"),
            ("=0.1+0.2", "0.3"),
            ("=1/3", "0.3333333333"),
            ("=((((7))))", "7"),
        ];
        for (f, want) in cases {
            assert_eq!(calc(f).display(), want, "{f}");
        }
    }

    #[test]
    fn references_and_ranges_read_other_cells() {
        let v = sheet(&[
            ("A1", "10"),
            ("A2", "20"),
            ("A3", "30"),
            ("B1", "=A1*2"),
            ("B2", "=SUM(A1:A3)"),
            ("B3", "=AVERAGE(A1:A3)"),
            ("B4", "=MIN(A1:A3, 5)"),
            ("B5", "=MAX(A3:A1)"),
            ("B6", "=sum(a1, a2) + max(1, 2, 3)"),
            ("B7", "=C9"),
            ("B8", "=SUM(A1:A3, B1) / 2"),
            ("B9", "=SUM()"),
        ]);
        let got: Vec<String> = (1..=9).map(|r| at(&v, &format!("B{r}"))).collect();
        assert_eq!(got, ["20", "60", "20", "5", "30", "33", "0", "40", "0"]);
    }

    #[test]
    fn functions_skip_text_but_arithmetic_rejects_it() {
        let v = sheet(&[
            ("A1", "4"),
            ("A2", "apples"),
            ("A3", "6"),
            ("B1", "=SUM(A1:A4)"),
            ("B2", "=AVERAGE(A1:A3)"),
            ("B3", "=A1+A2"),
            ("B4", "=A2"),
            ("B5", "=AVERAGE(C1:C5)"),
            ("B6", "=MIN(C1:C5)"),
        ]);
        assert_eq!(at(&v, "B1"), "10");
        assert_eq!(at(&v, "B2"), "5");
        assert_eq!(at(&v, "B3"), "#VALUE!");
        assert_eq!(at(&v, "B4"), "apples");
        assert_eq!(at(&v, "B5"), "#DIV/0!");
        assert_eq!(at(&v, "B6"), "0");
    }

    #[test]
    fn errors_have_their_own_codes() {
        let cases = [
            ("=1/0", "#DIV/0!"),
            ("=1/(2-2)", "#DIV/0!"),
            ("=A0", "#REF!"),
            ("=AA101", "#REF!"),
            ("=SUM(A1:A500)", "#REF!"),
            ("=FOO(1)", "#NAME?"),
            ("=total", "#NAME?"),
            ("=", "#ERROR!"),
            ("=1+", "#ERROR!"),
            ("=(1", "#ERROR!"),
            ("=1)", "#ERROR!"),
            ("=1 2", "#ERROR!"),
            ("=1.2.3", "#ERROR!"),
            ("=SUM(1,", "#ERROR!"),
            ("=A1:", "#ERROR!"),
            ("=2 & 3", "#ERROR!"),
            ("=A1:B2", "#VALUE!"),
        ];
        for (f, want) in cases {
            assert_eq!(calc(f).display(), want, "{f}");
        }
    }

    #[test]
    fn errors_propagate_to_dependents() {
        let v = sheet(&[("A1", "=1/0"), ("A2", "=A1+1"), ("A3", "=SUM(A1:A2)"), ("A4", "=MAX(B1, A1)")]);
        for name in ["A2", "A3", "A4"] {
            assert_eq!(at(&v, name), "#DIV/0!", "{name}");
        }
    }

    #[test]
    fn cycles_are_marked_and_spread_but_bystanders_are_fine() {
        let v = sheet(&[
            ("A1", "=A1+1"),
            ("B1", "=C1"),
            ("C1", "=D1*2"),
            ("D1", "=B1"),
            ("E1", "=SUM(B1:D1)"),
            ("F1", "5"),
            ("G1", "=F1*2"),
            ("H1", "=SUM(A2:A9)"),
            ("A5", "=H1"),
        ]);
        for name in ["A1", "B1", "C1", "D1", "E1", "H1", "A5"] {
            assert_eq!(at(&v, name), "#CYCLE!", "{name}");
        }
        assert_eq!(at(&v, "G1"), "10");
    }

    #[test]
    fn recalculation_is_dependency_ordered_and_iterative() {
        // A chain 5000 long, entered bottom-up so input order cannot help.
        let mut map = BTreeMap::new();
        map.insert((0, 0), "1".to_string());
        for row in (1..5000).rev() {
            map.insert((row, 0), format!("=A{row}+1"));
        }
        map.insert((0, 1), "=SUM(A1:A5000)".to_string());
        let v = recalc(&map, (5000, 2));
        assert_eq!(v[&(4999, 0)].display(), "5000");
        assert_eq!(v[&(0, 1)].display(), "12502500");
    }

    #[test]
    fn formula_results_feed_wide_ranges() {
        // More formula cells than the range is wide, and the other way round.
        let mut map: BTreeMap<Cell, String> = (0..50).map(|r| ((r, 0), format!("={}", r + 1))).collect();
        map.insert((0, 1), "=SUM(A1:A2)".to_string());
        map.insert((1, 1), "=SUM(A1:A10000)".to_string());
        let v = recalc(&map, (10_000, 4));
        assert_eq!(v[&(0, 1)].display(), "3");
        assert_eq!(v[&(1, 1)].display(), "1275");
    }

    #[test]
    fn literals_and_number_formatting() {
        assert_eq!(literal(" 42 "), Value::Number(42.0));
        assert_eq!(literal("1e3"), Value::Number(1000.0));
        assert_eq!(literal("inf"), Value::Text("inf".into()));
        assert_eq!(literal("NaN"), Value::Text("NaN".into()));
        assert_eq!(literal("  "), Value::Empty);
        assert_eq!(format_number(-0.0), "0");
        assert_eq!(format_number(1_250_000.0), "1250000");
        assert_eq!(format_number(-2.5), "-2.5");
        assert_eq!(format_number(1e-12), "0");
        assert_eq!(calc("=B1").display(), "0", "a blank reference is 0");
    }
}
