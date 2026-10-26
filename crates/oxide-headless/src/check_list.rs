//! Check lists: the state and summary line for a set of automated checks.

/// One check's state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckState {
    /// Passed.
    Pass,
    /// Failed.
    Fail,
    /// Skipped.
    Skip,
    /// Still running.
    Running,
}

impl CheckState {
    /// The `data-state` value.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            CheckState::Pass => "pass",
            CheckState::Fail => "fail",
            CheckState::Skip => "skip",
            CheckState::Running => "running",
        }
    }
}

/// One named check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Check {
    /// Name, such as "Unit tests".
    pub name: String,
    /// See [`CheckState`].
    pub state: CheckState,
}

impl Check {
    /// A named check in `state`.
    #[must_use]
    pub fn new(name: impl Into<String>, state: CheckState) -> Self {
        Self { name: name.into(), state }
    }
}

/// The overall state: Fail if any failed, else Running if any is running, else Pass.
#[must_use]
pub fn overall(checks: &[Check]) -> CheckState {
    if checks.iter().any(|c| c.state == CheckState::Fail) {
        CheckState::Fail
    } else if checks.iter().any(|c| c.state == CheckState::Running) {
        CheckState::Running
    } else {
        CheckState::Pass
    }
}

/// The summary line: "All 6 checks passed", "Checks running" or "4 of 6 checks passed".
#[must_use]
pub fn summary(checks: &[Check]) -> String {
    let passed = checks.iter().filter(|c| c.state == CheckState::Pass).count();
    match overall(checks) {
        CheckState::Running => "Checks running".to_string(),
        _ if passed == checks.len() => format!("All {} checks passed", checks.len()),
        _ => format!("{passed} of {} checks passed", checks.len()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn checks(states: &[CheckState]) -> Vec<Check> {
        states.iter().enumerate().map(|(i, s)| Check::new(format!("c{i}"), *s)).collect()
    }

    #[test]
    fn all_pass() {
        let c = checks(&[CheckState::Pass; 6]);
        assert_eq!((overall(&c), summary(&c).as_str()), (CheckState::Pass, "All 6 checks passed"));
    }

    #[test]
    fn a_failure_wins_over_running() {
        use CheckState::{Fail, Pass, Running, Skip};
        let c = checks(&[Pass, Pass, Pass, Pass, Fail, Running]);
        assert_eq!(overall(&c), Fail);
        assert_eq!(summary(&c), "4 of 6 checks passed");
        let r = checks(&[Pass, Running, Skip]);
        assert_eq!((overall(&r), summary(&r).as_str()), (Running, "Checks running"));
        let s = checks(&[Pass, Skip]);
        assert_eq!((overall(&s), summary(&s).as_str()), (Pass, "1 of 2 checks passed"));
    }
}
