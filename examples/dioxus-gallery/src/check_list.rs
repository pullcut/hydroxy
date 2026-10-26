use dioxus::prelude::*;
use oxide_dioxus::check_list::{Check, CheckList, CheckState};

#[component]
pub fn CheckListDemo() -> Element {
    let finished = vec![
        Check::new("Lint", CheckState::Pass),
        Check::new("Unit tests", CheckState::Pass),
        Check::new("Accessibility", CheckState::Fail),
        Check::new("Visual diff", CheckState::Skip),
    ];
    let running = vec![
        Check::new("Lint", CheckState::Pass),
        Check::new("Unit tests", CheckState::Running),
        Check::new("Accessibility", CheckState::Running),
    ];
    rsx! {
        div { style: "display: grid; gap: var(--space-5); max-width: 420px;",
            CheckList { checks: finished }
            CheckList { checks: running }
        }
    }
}
