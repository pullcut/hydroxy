//! Filter builder connector. Anatomy: root, rules, rule, field, op, value,
//! remove, footer, add, join. Every part carries `data-scope="filter-builder"`
//! and its `data-part`. The root is `role="group"`; each rule is a `role="group"`
//! named "Condition N".

pub use hydroxy_machines::filter_builder::*;

/// The `data-scope` value on every part.
pub const SCOPE: &str = "filter-builder";

/// The root group's accessible name.
pub const LABEL: &str = "Filter";

/// Accessible names for the controls in rule number `index` (0-based).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleLabels {
    /// The rule group.
    pub group: String,
    /// The field menu.
    pub field: String,
    /// The operator menu.
    pub op: String,
    /// The value input.
    pub value: String,
    /// The remove button.
    pub remove: String,
}

/// Labels for rule `index`.
#[must_use]
pub fn rule_labels(index: usize) -> RuleLabels {
    let n = index + 1;
    RuleLabels {
        group: format!("Condition {n}"),
        field: format!("Field, condition {n}"),
        op: format!("Operator, condition {n}"),
        value: format!("Value, condition {n}"),
        remove: format!("Remove condition {n}"),
    }
}

/// (value, label) pairs for the operator menu of a field kind.
#[must_use]
pub fn op_options(kind: &FieldKind) -> Vec<(String, String)> {
    kind.ops().iter().map(|o| (o.as_str().to_string(), o.label().to_string())).collect()
}

/// (value, label) pairs for the AND / OR switch.
#[must_use]
pub fn join_options() -> Vec<(String, String)> {
    vec![("and".into(), "Match all".into()), ("or".into(), "Match any".into())]
}

/// The join for a menu value.
#[must_use]
pub fn parse_join(value: &str) -> Join {
    if value == "or" { Join::Or } else { Join::And }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn labels_count_from_one() {
        assert_eq!(rule_labels(0).group, "Condition 1");
        assert_eq!(rule_labels(2).remove, "Remove condition 3");
    }

    #[test]
    fn ops_follow_kind_and_joins_round_trip() {
        let ops = op_options(&FieldKind::Number);
        assert_eq!(ops[0], ("eq".to_string(), "is".to_string()));
        assert!(!ops.iter().any(|(v, _)| v == "contains"));
        assert_eq!(parse_join("or"), Join::Or);
        assert_eq!(parse_join("and"), Join::And);
    }
}
