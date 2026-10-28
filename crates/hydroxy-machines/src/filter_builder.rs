//! Filter builder owns a list of rules (field, operator, value) joined by AND
//! or OR. The model is plain data: [`Filter`] holds [`Rule`]s, each [`Op`]
//! knows how to test a value, and [`Filter::matches`] runs the lot against a
//! record. The machine only edits the filter and reports every change.

use hydroxy_statechart::{Machine, Transition};

/// One choice for a [`FieldKind::Choice`] field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Choice {
    /// Stored in the rule.
    pub value: String,
    /// Shown to the user.
    pub label: String,
}

impl Choice {
    /// A choice.
    #[must_use]
    pub fn new(value: impl Into<String>, label: impl Into<String>) -> Self {
        Self { value: value.into(), label: label.into() }
    }
}

/// What a field holds. It decides which operators are offered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FieldKind {
    /// Free text.
    Text,
    /// A number.
    Number,
    /// One of a fixed set.
    Choice(Vec<Choice>),
}

impl FieldKind {
    /// The operators that make sense for this kind, in menu order.
    #[must_use]
    pub fn ops(&self) -> &'static [Op] {
        match self {
            FieldKind::Text => &[Op::Contains, Op::NotContains, Op::Is, Op::IsNot, Op::StartsWith, Op::IsEmpty, Op::IsNotEmpty],
            FieldKind::Number => &[Op::Eq, Op::Ne, Op::Gt, Op::Gte, Op::Lt, Op::Lte],
            FieldKind::Choice(_) => &[Op::Is, Op::IsNot],
        }
    }

    fn default_value(&self) -> String {
        match self {
            FieldKind::Choice(choices) => choices.first().map(|c| c.value.clone()).unwrap_or_default(),
            _ => String::new(),
        }
    }
}

/// A field a rule can test.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Field {
    /// Unique key, stored in the rule.
    pub id: String,
    /// Shown in the field menu.
    pub label: String,
    /// What it holds.
    pub kind: FieldKind,
}

impl Field {
    /// A field.
    #[must_use]
    pub fn new(id: impl Into<String>, label: impl Into<String>, kind: FieldKind) -> Self {
        Self { id: id.into(), label: label.into(), kind }
    }
}

/// How a rule compares a field to its value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Op {
    /// Text contains the value, ignoring case.
    Contains,
    /// Text does not contain the value.
    NotContains,
    /// Equals the value (text ignores case).
    Is,
    /// Does not equal the value.
    IsNot,
    /// Text starts with the value.
    StartsWith,
    /// Text is blank. Takes no value.
    IsEmpty,
    /// Text is not blank. Takes no value.
    IsNotEmpty,
    /// Number equals.
    Eq,
    /// Number differs.
    Ne,
    /// Number is greater.
    Gt,
    /// Number is greater or equal.
    Gte,
    /// Number is less.
    Lt,
    /// Number is less or equal.
    Lte,
}

impl Op {
    /// Stable key, safe for `<option value>`.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Op::Contains => "contains",
            Op::NotContains => "not-contains",
            Op::Is => "is",
            Op::IsNot => "is-not",
            Op::StartsWith => "starts-with",
            Op::IsEmpty => "is-empty",
            Op::IsNotEmpty => "is-not-empty",
            Op::Eq => "eq",
            Op::Ne => "ne",
            Op::Gt => "gt",
            Op::Gte => "gte",
            Op::Lt => "lt",
            Op::Lte => "lte",
        }
    }

    /// The op for a key from [`Op::as_str`].
    #[must_use]
    pub fn parse(key: &str) -> Option<Self> {
        const ALL: [Op; 13] = [
            Op::Contains,
            Op::NotContains,
            Op::Is,
            Op::IsNot,
            Op::StartsWith,
            Op::IsEmpty,
            Op::IsNotEmpty,
            Op::Eq,
            Op::Ne,
            Op::Gt,
            Op::Gte,
            Op::Lt,
            Op::Lte,
        ];
        ALL.into_iter().find(|o| o.as_str() == key)
    }

    /// Text for the operator menu.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Op::Contains => "contains",
            Op::NotContains => "does not contain",
            Op::Is | Op::Eq => "is",
            Op::IsNot | Op::Ne => "is not",
            Op::StartsWith => "starts with",
            Op::IsEmpty => "is empty",
            Op::IsNotEmpty => "is not empty",
            Op::Gt => "is greater than",
            Op::Gte => "is at least",
            Op::Lt => "is less than",
            Op::Lte => "is at most",
        }
    }

    /// Whether the rule needs a value.
    #[must_use]
    pub const fn needs_value(self) -> bool {
        !matches!(self, Op::IsEmpty | Op::IsNotEmpty)
    }

    /// Test `actual` against `expected`. Numbers that don't parse never match.
    #[must_use]
    pub fn apply(self, actual: &str, expected: &str) -> bool {
        let (a, e) = (actual.to_lowercase(), expected.to_lowercase());
        let num = |f: fn(f64, f64) -> bool| match (actual.trim().parse::<f64>(), expected.trim().parse::<f64>()) {
            (Ok(a), Ok(e)) => f(a, e),
            _ => false,
        };
        match self {
            Op::Contains => a.contains(&e),
            Op::NotContains => !a.contains(&e),
            Op::Is => a == e,
            Op::IsNot => a != e,
            Op::StartsWith => a.starts_with(&e),
            Op::IsEmpty => actual.trim().is_empty(),
            Op::IsNotEmpty => !actual.trim().is_empty(),
            Op::Eq => num(|a, e| (a - e).abs() < f64::EPSILON),
            Op::Ne => num(|a, e| (a - e).abs() >= f64::EPSILON),
            Op::Gt => num(|a, e| a > e),
            Op::Gte => num(|a, e| a >= e),
            Op::Lt => num(|a, e| a < e),
            Op::Lte => num(|a, e| a <= e),
        }
    }
}

/// How the rules combine.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Join {
    /// Every rule must match.
    #[default]
    And,
    /// Any rule may match.
    Or,
}

impl Join {
    /// Stable key.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Join::And => "and",
            Join::Or => "or",
        }
    }
}

/// One condition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rule {
    /// Unique among the filter's rules. Stable while the rule is edited.
    pub id: u32,
    /// A [`Field::id`].
    pub field: String,
    /// The comparison.
    pub op: Op,
    /// What to compare with. Empty for operators that take none.
    pub value: String,
}

/// The finished filter: plain data the owner can store or send.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Filter {
    /// How the rules combine.
    pub join: Join,
    /// The rules, top to bottom.
    pub rules: Vec<Rule>,
}

impl Filter {
    /// Whether a record passes. `get` returns the record's text for a field id.
    /// An empty filter passes everything; a missing field reads as empty.
    pub fn matches(&self, get: impl Fn(&str) -> Option<String>) -> bool {
        let test = |r: &Rule| r.op.apply(&get(&r.field).unwrap_or_default(), &r.value);
        match self.join {
            _ if self.rules.is_empty() => true,
            Join::And => self.rules.iter().all(test),
            Join::Or => self.rules.iter().any(test),
        }
    }
}

/// Props for [`FilterBuilderMachine`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FilterBuilderInput {
    /// The fields rules can use. The first is the default for a new rule.
    pub fields: Vec<Field>,
    /// Controlled filter. `Some` at start fixes controlled mode.
    pub filter: Option<Filter>,
    /// Uncontrolled initial filter.
    pub default_filter: Filter,
}

/// What [`FilterBuilderMachine`] remembers between events.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FilterBuilderState {
    /// The current filter.
    pub filter: Filter,
    /// Fixed at start: the owner holds the filter.
    pub controlled: bool,
}

/// Inputs to [`FilterBuilderMachine`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FilterBuilderEvent {
    /// Add a rule on the first field.
    Add,
    /// Remove a rule.
    Remove(u32),
    /// Point a rule at another field. Resets its operator and value to suit.
    SetField {
        /// The rule.
        rule: u32,
        /// The new field id.
        field: String,
    },
    /// Change a rule's operator.
    SetOp {
        /// The rule.
        rule: u32,
        /// The new operator. Ignored if the field doesn't offer it.
        op: Op,
    },
    /// Change a rule's value.
    SetValue {
        /// The rule.
        rule: u32,
        /// The new value.
        value: String,
    },
    /// Switch AND / OR.
    SetJoin(Join),
}

/// Side effects from [`FilterBuilderMachine`] for the adapter to run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FilterBuilderCommand {
    /// The filter changed (or should, when controlled).
    Change(Filter),
}

/// Filter editing state.
#[derive(Debug, Clone, Copy)]
pub struct FilterBuilderMachine;

fn fresh_rule(fields: &[Field], filter: &Filter) -> Option<Rule> {
    let id = filter.rules.iter().map(|r| r.id).max().map_or(0, |m| m + 1);
    let f = fields.first()?;
    Some(Rule { id, field: f.id.clone(), op: *f.kind.ops().first()?, value: f.kind.default_value() })
}

impl Machine for FilterBuilderMachine {
    type Input = FilterBuilderInput;
    type State = FilterBuilderState;
    type Event = FilterBuilderEvent;
    type Command = FilterBuilderCommand;

    fn initial(input: &FilterBuilderInput) -> FilterBuilderState {
        FilterBuilderState {
            filter: input.filter.clone().unwrap_or_else(|| input.default_filter.clone()),
            controlled: input.filter.is_some(),
        }
    }

    fn transition(
        state: &FilterBuilderState,
        event: FilterBuilderEvent,
        input: &FilterBuilderInput,
    ) -> Transition<FilterBuilderState, FilterBuilderCommand> {
        let mut filter = state.filter.clone();
        let rule_mut = |filter: &mut Filter, id: u32| filter.rules.iter().position(|r| r.id == id);
        let changed = match event {
            FilterBuilderEvent::Add => fresh_rule(&input.fields, &filter).map(|r| filter.rules.push(r)).is_some(),
            FilterBuilderEvent::Remove(id) => {
                let before = filter.rules.len();
                filter.rules.retain(|r| r.id != id);
                filter.rules.len() != before
            }
            FilterBuilderEvent::SetField { rule, field } => {
                match (rule_mut(&mut filter, rule), input.fields.iter().find(|f| f.id == field)) {
                    (Some(i), Some(f)) if filter.rules[i].field != field => {
                        let Some(op) = f.kind.ops().first().copied() else { return Transition::stay() };
                        filter.rules[i] = Rule { id: rule, field, op, value: f.kind.default_value() };
                        true
                    }
                    _ => false,
                }
            }
            FilterBuilderEvent::SetOp { rule, op } => {
                let allowed = |r: &Rule| {
                    input.fields.iter().find(|f| f.id == r.field).is_some_and(|f| f.kind.ops().contains(&op))
                };
                match rule_mut(&mut filter, rule) {
                    Some(i) if filter.rules[i].op != op && allowed(&filter.rules[i]) => {
                        filter.rules[i].op = op;
                        if !op.needs_value() {
                            filter.rules[i].value.clear();
                        }
                        true
                    }
                    _ => false,
                }
            }
            FilterBuilderEvent::SetValue { rule, value } => match rule_mut(&mut filter, rule) {
                Some(i) if filter.rules[i].value != value => {
                    filter.rules[i].value = value;
                    true
                }
                _ => false,
            },
            FilterBuilderEvent::SetJoin(join) => {
                let differs = filter.join != join;
                filter.join = join;
                differs
            }
        };
        if !changed {
            return Transition::stay();
        }
        let mut next = state.clone();
        if !state.controlled {
            next.filter = filter.clone();
        }
        Transition::with(next, vec![FilterBuilderCommand::Change(filter)])
    }

    fn sync(state: &FilterBuilderState, input: &FilterBuilderInput) -> Option<FilterBuilderState> {
        let mut next = state.clone();
        if state.controlled {
            next.filter = input.filter.clone().unwrap_or_default();
        }
        hydroxy_statechart::changed(state, next)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fields() -> Vec<Field> {
        vec![
            Field::new("title", "Title", FieldKind::Text),
            Field::new("score", "Score", FieldKind::Number),
            Field::new("status", "Status", FieldKind::Choice(vec![Choice::new("open", "Open"), Choice::new("done", "Done")])),
        ]
    }

    fn input() -> FilterBuilderInput {
        FilterBuilderInput { fields: fields(), ..FilterBuilderInput::default() }
    }

    fn run(state: &FilterBuilderState, event: FilterBuilderEvent, input: &FilterBuilderInput) -> hydroxy_statechart::Step<FilterBuilderState, FilterBuilderCommand> {
        FilterBuilderMachine::transition(state, event, input).resolve(state)
    }

    fn with_rule() -> (FilterBuilderState, FilterBuilderInput) {
        let i = input();
        let s = run(&FilterBuilderMachine::initial(&i), FilterBuilderEvent::Add, &i).state;
        (s, i)
    }

    #[test]
    fn add_uses_the_first_field_and_its_first_op() {
        let (s, _) = with_rule();
        assert_eq!(s.filter.rules, [Rule { id: 0, field: "title".into(), op: Op::Contains, value: String::new() }]);
    }

    #[test]
    fn ids_never_repeat_after_a_remove() {
        let (s, i) = with_rule();
        let s = run(&s, FilterBuilderEvent::Add, &i).state;
        let s = run(&s, FilterBuilderEvent::Remove(0), &i).state;
        let s = run(&s, FilterBuilderEvent::Add, &i).state;
        let ids: Vec<u32> = s.filter.rules.iter().map(|r| r.id).collect();
        assert_eq!(ids, [1, 2]);
    }

    #[test]
    fn changing_field_resets_op_and_value() {
        let (s, i) = with_rule();
        let s = run(&s, FilterBuilderEvent::SetValue { rule: 0, value: "x".into() }, &i).state;
        let s = run(&s, FilterBuilderEvent::SetField { rule: 0, field: "status".into() }, &i).state;
        assert_eq!(s.filter.rules[0].op, Op::Is);
        assert_eq!(s.filter.rules[0].value, "open");
        let s = run(&s, FilterBuilderEvent::SetField { rule: 0, field: "score".into() }, &i).state;
        assert_eq!(s.filter.rules[0].op, Op::Eq);
    }

    #[test]
    fn operators_follow_the_field_type() {
        let (s, i) = with_rule();
        // Numbers can't "contain".
        let n = run(&s, FilterBuilderEvent::SetField { rule: 0, field: "score".into() }, &i).state;
        let step = run(&n, FilterBuilderEvent::SetOp { rule: 0, op: Op::Contains }, &i);
        assert!(step.commands.is_empty());
        let step = run(&n, FilterBuilderEvent::SetOp { rule: 0, op: Op::Gt }, &i);
        assert_eq!(step.state.filter.rules[0].op, Op::Gt);
    }

    #[test]
    fn valueless_ops_clear_the_value() {
        let (s, i) = with_rule();
        let s = run(&s, FilterBuilderEvent::SetValue { rule: 0, value: "x".into() }, &i).state;
        let s = run(&s, FilterBuilderEvent::SetOp { rule: 0, op: Op::IsEmpty }, &i).state;
        assert_eq!(s.filter.rules[0].value, "");
    }

    #[test]
    fn unchanged_events_report_nothing() {
        let (s, i) = with_rule();
        assert!(run(&s, FilterBuilderEvent::SetJoin(Join::And), &i).commands.is_empty());
        assert!(run(&s, FilterBuilderEvent::Remove(99), &i).commands.is_empty());
        assert!(run(&FilterBuilderMachine::initial(&FilterBuilderInput::default()), FilterBuilderEvent::Add, &FilterBuilderInput::default()).commands.is_empty());
    }

    #[test]
    fn controlled_reports_without_changing() {
        let i = FilterBuilderInput { filter: Some(Filter::default()), ..input() };
        let s = FilterBuilderMachine::initial(&i);
        let step = run(&s, FilterBuilderEvent::Add, &i);
        assert!(step.state.filter.rules.is_empty());
        let FilterBuilderCommand::Change(f) = &step.commands[0];
        assert_eq!(f.rules.len(), 1);
    }

    fn record(field: &str) -> Option<String> {
        match field {
            "title" => Some("Weekly digest".into()),
            "score" => Some("12".into()),
            "status" => Some("open".into()),
            _ => None,
        }
    }

    fn rule(field: &str, op: Op, value: &str) -> Rule {
        Rule { id: 0, field: field.into(), op, value: value.into() }
    }

    #[test]
    fn matches_and_or() {
        let hit = rule("title", Op::Contains, "DIGEST");
        let miss = rule("score", Op::Gt, "20");
        let and = Filter { join: Join::And, rules: vec![hit.clone(), miss.clone()] };
        let or = Filter { join: Join::Or, rules: vec![hit, miss] };
        assert!(!and.matches(record));
        assert!(or.matches(record));
        assert!(Filter::default().matches(record));
    }

    #[test]
    fn ops_compare_sensibly() {
        assert!(Op::Gt.apply("12", "9"));
        assert!(!Op::Gt.apply("abc", "9"));
        assert!(Op::StartsWith.apply("Weekly", "week"));
        assert!(Op::IsEmpty.apply("  ", ""));
        assert!(Op::IsNot.apply("open", "done"));
        assert_eq!(Op::parse("starts-with"), Some(Op::StartsWith));
        assert_eq!(Op::parse("nope"), None);
        let f = Filter { join: Join::And, rules: vec![rule("missing", Op::IsEmpty, "")] };
        assert!(f.matches(record));
    }
}
