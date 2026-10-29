//! Resizable panels connector. Anatomy: root, panel, separator. Every part
//! carries `data-scope="resizable"`, its `data-part` and `data-orientation`.
//! Panels carry `data-state="collapsed|expanded"`; separators `data-dragging`.

pub use hydroxy_machines::resizable::*;

pub use crate::Orientation;

/// `data-scope` for every part.
pub const SCOPE: &str = "resizable";

/// Panel `i`'s element id.
#[must_use]
pub fn panel_id(root: &str, i: usize) -> String {
    format!("{root}-panel-{i}")
}

/// Attributes for divider `d`, a focusable `role="separator"`.
#[derive(Debug, Clone, PartialEq)]
pub struct SeparatorAttrs {
    /// `aria-controls`: the panel Enter collapses and the value describes.
    pub aria_controls: String,
    /// `aria-orientation`: the separator's own axis, across the group's.
    pub aria_orientation: &'static str,
    /// That panel's size, rounded percent.
    pub aria_valuenow: i64,
    /// Zero when collapsible, else the panel's minimum.
    pub aria_valuemin: i64,
    /// The panel's maximum.
    pub aria_valuemax: i64,
}

#[allow(clippy::cast_possible_truncation, reason = "percentages fit in i64")]
fn pct(v: f64) -> i64 {
    v.round() as i64
}

/// Attributes for divider `d` of the instance with id prefix `root`.
#[must_use]
pub fn separator_attrs(
    root: &str,
    d: usize,
    orientation: Orientation,
    state: &ResizableState,
    input: &ResizableInput,
) -> SeparatorAttrs {
    let p = controlled_panel(&input.panels, d);
    let spec = input.panels.get(p).copied().unwrap_or_default();
    SeparatorAttrs {
        aria_controls: panel_id(root, p),
        aria_orientation: match orientation {
            Orientation::Horizontal => "vertical",
            Orientation::Vertical => "horizontal",
        },
        aria_valuenow: pct(state.sizes.value.get(p).copied().unwrap_or_default()),
        aria_valuemin: if spec.collapsible { 0 } else { pct(spec.min) },
        aria_valuemax: pct(spec.max),
    }
}

/// `data-state` for panel `i`.
#[must_use]
pub fn panel_state(state: &ResizableState, i: usize) -> &'static str {
    if state.sizes.value.get(i).is_some_and(|s| *s <= 0.0) { "collapsed" } else { "expanded" }
}

/// Inline style for panel `i`: its size as a flex weight, so separators never overflow.
#[must_use]
pub fn panel_style(state: &ResizableState, i: usize) -> String {
    format!("flex: {} 1 0px;", state.sizes.value.get(i).copied().unwrap_or_default())
}

/// The event for a key on divider `d`, if the key does anything.
#[must_use]
pub fn key_event(key: &str, d: usize, orientation: Orientation) -> Option<ResizableEvent> {
    match (key, orientation) {
        ("ArrowRight", Orientation::Horizontal) | ("ArrowDown", Orientation::Vertical) => {
            Some(ResizableEvent::Step { divider: d, grow: true })
        }
        ("ArrowLeft", Orientation::Horizontal) | ("ArrowUp", Orientation::Vertical) => {
            Some(ResizableEvent::Step { divider: d, grow: false })
        }
        ("Home", _) => Some(ResizableEvent::Home(d)),
        ("End", _) => Some(ResizableEvent::End(d)),
        ("Enter", _) => Some(ResizableEvent::ToggleCollapse(d)),
        _ => None,
    }
}

/// Divider `d`'s position, percent from the group's start edge.
#[must_use]
pub fn divider_position(state: &ResizableState, d: usize) -> f64 {
    state.sizes.value.iter().take(d + 1).sum()
}

/// Where a drag puts a divider: where it started plus the pointer's travel,
/// converted from pixels to percent of the group's length.
#[must_use]
pub fn drag_position(origin: f64, travel_px: f64, length_px: f64) -> f64 {
    if length_px > 0.0 { origin + travel_px / length_px * 100.0 } else { origin }
}

/// The separator's accessible name.
#[must_use]
pub fn separator_label(panel_label: Option<&str>) -> String {
    panel_label.map_or_else(|| "Resize panel".to_string(), |l| format!("Resize {l}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use hydroxy_statechart::Controllable;

    #[test]
    fn separator_describes_the_controlled_panel() {
        let input = ResizableInput {
            panels: vec![
                PanelSpec::default(),
                PanelSpec { min: 20.0, max: 40.0, collapsible: true, default_size: None },
            ],
            ..Default::default()
        };
        let state = ResizableState { sizes: Controllable::new(vec![70.4, 29.6], false), ..Default::default() };
        let a = separator_attrs("r", 0, Orientation::Horizontal, &state, &input);
        assert_eq!((a.aria_controls.as_str(), a.aria_orientation), ("r-panel-1", "vertical"));
        assert_eq!((a.aria_valuenow, a.aria_valuemin, a.aria_valuemax), (30, 0, 40));
    }

    #[test]
    fn keys_follow_orientation() {
        assert_eq!(
            key_event("ArrowDown", 0, Orientation::Vertical),
            Some(ResizableEvent::Step { divider: 0, grow: true })
        );
        assert_eq!(key_event("ArrowDown", 0, Orientation::Horizontal), None);
        assert_eq!(key_event("Enter", 2, Orientation::Horizontal), Some(ResizableEvent::ToggleCollapse(2)));
        assert!((drag_position(30.0, 50.0, 200.0) - 55.0).abs() < 1e-9);
    }
}
