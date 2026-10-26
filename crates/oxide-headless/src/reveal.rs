//! Reveal connector. A single part, root, that carries `data-scope="reveal"`
//! and `data-state="hidden|shown"`. The styles do the animating, so the
//! content is fully visible with no stylesheet or script.

pub use oxide_machines::reveal::*;

/// The `data-scope` value.
pub const SCOPE: &str = "reveal";

/// `data-state` for the root.
#[must_use]
pub fn data_state(state: &RevealState) -> &'static str {
    if state.shown { "shown" } else { "hidden" }
}

/// The root's id, which the observer targets.
#[must_use]
pub fn root_id(root: &str) -> String {
    format!("{root}-root")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_follows_the_latch() {
        assert_eq!(data_state(&RevealState::default()), "hidden");
        assert_eq!(data_state(&RevealState { shown: true, ..RevealState::default() }), "shown");
    }
}
