//! Resizable panels: N panels in a row or column, sized in percent, with a
//! divider between each pair. A divider only moves the two panels beside it.

use hydroxy_statechart::{Machine, Transition};

/// One panel's limits, in percent of the group.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PanelSpec {
    /// Smallest size while expanded.
    pub min: f64,
    /// Largest size.
    pub max: f64,
    /// Can shrink to zero: dragged past half its `min`, Home, or Enter.
    pub collapsible: bool,
    /// Initial size. Panels without one share what is left.
    pub default_size: Option<f64>,
}

impl Default for PanelSpec {
    fn default() -> Self {
        Self { min: 0.0, max: 100.0, collapsible: false, default_size: None }
    }
}

/// Props for [`ResizableMachine`].
#[derive(Debug, Clone, PartialEq)]
pub struct ResizableInput {
    /// Panels in order.
    pub panels: Vec<PanelSpec>,
    /// Controlled sizes. `Some` at start fixes controlled mode.
    pub sizes: Option<Vec<f64>>,
    /// Keyboard step in percent.
    pub step: f64,
}

impl Default for ResizableInput {
    fn default() -> Self {
        Self { panels: Vec::new(), sizes: None, step: 5.0 }
    }
}

/// What [`ResizableMachine`] remembers between events.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ResizableState {
    /// Panel sizes in percent, summing to 100.
    pub sizes: Vec<f64>,
    /// Size to restore when a collapsed panel expands.
    pub restore: Vec<f64>,
    /// The divider being dragged.
    pub dragging: Option<usize>,
    /// Fixed at start: the owner holds `sizes`.
    pub controlled: bool,
}

/// Inputs to [`ResizableMachine`]. `usize` is the divider index: divider `i`
/// sits between panels `i` and `i + 1`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ResizableEvent {
    /// Pointer down on a divider.
    DragStart(usize),
    /// Pointer moved to `at` percent from the group's start edge.
    DragTo(f64),
    /// Pointer released.
    DragEnd,
    /// Arrow key: grow the panel before the divider by one step, or shrink it.
    Step {
        /// The divider.
        divider: usize,
        /// Grow the panel before (`true`) or shrink it.
        grow: bool,
    },
    /// Home: the panel before goes to its minimum, or collapses.
    Home(usize),
    /// End: the panel before goes to its maximum.
    End(usize),
    /// Enter: collapse or expand the divider's [`controlled_panel`].
    ToggleCollapse(usize),
}

/// Side effects from [`ResizableMachine`] for the adapter to run.
#[derive(Debug, Clone, PartialEq)]
pub enum ResizableCommand {
    /// The sizes changed (or should, when controlled).
    SizesChange(Vec<f64>),
}

/// Panel sizes and divider drags.
#[derive(Debug, Clone, Copy)]
pub struct ResizableMachine;

impl Machine for ResizableMachine {
    type Input = ResizableInput;
    type State = ResizableState;
    type Event = ResizableEvent;
    type Command = ResizableCommand;

    fn initial(input: &ResizableInput) -> ResizableState {
        let sizes = match &input.sizes {
            Some(sizes) if sizes.len() == input.panels.len() => sizes.clone(),
            _ => default_sizes(&input.panels),
        };
        let restore = input.panels.iter().zip(&sizes).map(|(p, s)| s.max(p.min)).collect();
        ResizableState { sizes, restore, dragging: None, controlled: input.sizes.is_some() }
    }

    fn transition(
        state: &ResizableState,
        event: ResizableEvent,
        input: &ResizableInput,
    ) -> Transition<ResizableState, ResizableCommand> {
        let panels = &input.panels;
        let valid = |d: usize| d + 1 < panels.len() && state.sizes.len() == panels.len();
        let sizes = match event {
            ResizableEvent::DragStart(d) if valid(d) => {
                return Transition::to(ResizableState { dragging: Some(d), ..state.clone() });
            }
            ResizableEvent::DragEnd => return Transition::to(ResizableState { dragging: None, ..state.clone() }),
            ResizableEvent::DragTo(at) => match state.dragging {
                Some(d) if valid(d) => {
                    let start: f64 = state.sizes[..d].iter().sum();
                    move_divider(panels, &state.sizes, d, at - start)
                }
                _ => return Transition::stay(),
            },
            ResizableEvent::Step { divider: d, grow } if valid(d) => {
                let (a, b) = (state.sizes[d], state.sizes[d + 1]);
                let mut target = if grow { a + input.step } else { a - input.step };
                // Stepping out of a collapsed panel opens it at its minimum.
                if grow && a <= 0.0 {
                    target = target.max(panels[d].min);
                }
                if !grow && b <= 0.0 {
                    target = target.min(a + b - panels[d + 1].min);
                }
                move_divider(panels, &state.sizes, d, target)
            }
            ResizableEvent::Home(d) if valid(d) => move_divider(panels, &state.sizes, d, 0.0),
            ResizableEvent::End(d) if valid(d) => {
                move_divider(panels, &state.sizes, d, state.sizes[d] + state.sizes[d + 1])
            }
            ResizableEvent::ToggleCollapse(d) if valid(d) => {
                let p = controlled_panel(panels, d);
                if !panels[p].collapsible {
                    return Transition::stay();
                }
                let total = state.sizes[d] + state.sizes[d + 1];
                let size = if state.sizes[p] <= 0.0 { state.restore[p] } else { 0.0 };
                let before = if p == d { size } else { total - size };
                move_divider(panels, &state.sizes, d, before)
            }
            _ => return Transition::stay(),
        };
        if sizes == state.sizes {
            return Transition::stay();
        }
        let mut next = state.clone();
        for (k, (&old, &new)) in state.sizes.iter().zip(&sizes).enumerate() {
            if old > 0.0 && new <= 0.0 {
                next.restore[k] = old;
            }
        }
        if !state.controlled {
            next.sizes.clone_from(&sizes);
        }
        Transition::with(next, vec![ResizableCommand::SizesChange(sizes)])
    }

    fn sync(state: &ResizableState, input: &ResizableInput) -> Option<ResizableState> {
        if state.sizes.len() != input.panels.len() {
            return Some(ResizableState { controlled: state.controlled, ..Self::initial(input) });
        }
        let mut next = state.clone();
        if let (true, Some(sizes)) = (state.controlled, &input.sizes)
            && sizes.len() == input.panels.len()
        {
            next.sizes.clone_from(sizes);
        }
        hydroxy_statechart::changed(state, next)
    }
}

/// Sizes from each panel's `default_size`; the rest share the remainder. Scaled to sum to 100.
#[must_use]
pub fn default_sizes(panels: &[PanelSpec]) -> Vec<f64> {
    let given: f64 = panels.iter().filter_map(|p| p.default_size).sum();
    let free = panels.iter().filter(|p| p.default_size.is_none()).count();
    #[allow(clippy::cast_precision_loss, reason = "panel counts are tiny")]
    let share = if free > 0 { (100.0 - given).max(0.0) / free as f64 } else { 0.0 };
    let sizes: Vec<f64> = panels.iter().map(|p| p.default_size.unwrap_or(share)).collect();
    let sum: f64 = sizes.iter().sum();
    if sum > 0.0 { sizes.iter().map(|s| s * 100.0 / sum).collect() } else { sizes }
}

/// The panel a divider's Enter key collapses and its `aria-valuenow` reports:
/// the one before, unless only the one after is collapsible.
#[must_use]
pub fn controlled_panel(panels: &[PanelSpec], divider: usize) -> usize {
    match (panels.get(divider), panels.get(divider + 1)) {
        (Some(before), Some(after)) if !before.collapsible && after.collapsible => divider + 1,
        _ => divider,
    }
}

/// Move divider `d` so the panel before it is `target` percent, within both
/// panels' limits. A collapsible panel pushed below half its `min` snaps to zero.
#[must_use]
pub fn move_divider(panels: &[PanelSpec], sizes: &[f64], d: usize, target: f64) -> Vec<f64> {
    let (a, b) = (&panels[d], &panels[d + 1]);
    let total = sizes[d] + sizes[d + 1];
    let lo = a.min.max(total - b.max);
    let hi = a.max.min(total - b.min);
    let before = if a.collapsible && target < a.min / 2.0 && total <= b.max {
        0.0
    } else if b.collapsible && total - target < b.min / 2.0 && total <= a.max {
        total
    } else if lo <= hi {
        target.clamp(lo, hi)
    } else {
        return sizes.to_vec();
    };
    let mut next = sizes.to_vec();
    next[d] = before;
    next[d + 1] = total - before;
    next
}

#[cfg(test)]
mod tests {
    use super::*;

    fn panel(min: f64, max: f64, collapsible: bool) -> PanelSpec {
        PanelSpec { min, max, collapsible, default_size: None }
    }

    fn input(panels: Vec<PanelSpec>) -> ResizableInput {
        ResizableInput { panels, ..Default::default() }
    }

    fn close(a: &[f64], b: &[f64]) -> bool {
        a.len() == b.len() && a.iter().zip(b).all(|(x, y)| (x - y).abs() < 1e-9)
    }

    #[test]
    fn defaults_share_the_remainder_and_sum_to_100() {
        let panels =
            [PanelSpec { default_size: Some(20.0), ..Default::default() }, PanelSpec::default(), PanelSpec::default()];
        assert!(close(&default_sizes(&panels), &[20.0, 40.0, 40.0]));
        let over = [
            PanelSpec { default_size: Some(30.0), ..Default::default() },
            PanelSpec { default_size: Some(90.0), ..Default::default() },
        ];
        assert!(close(&default_sizes(&over), &[25.0, 75.0]));
    }

    #[test]
    fn drag_moves_only_neighbours_within_limits() {
        let input = input(vec![panel(10.0, 100.0, false), panel(20.0, 60.0, false), panel(10.0, 100.0, false)]);
        let s = ResizableMachine::initial(&ResizableInput { sizes: None, ..input.clone() });
        let s = ResizableState { sizes: vec![30.0, 40.0, 30.0], ..s };
        let s = ResizableMachine::transition(&s, ResizableEvent::DragStart(1), &input).resolve(&s).state;
        // Divider 1 sits at 70; drag to 95 → panel 2 can only shrink to 10.
        let t = ResizableMachine::transition(&s, ResizableEvent::DragTo(95.0), &input).resolve(&s);
        assert!(close(&t.state.sizes, &[30.0, 60.0, 10.0]));
        assert_eq!(t.commands.len(), 1);
        let s = ResizableMachine::transition(&t.state, ResizableEvent::DragEnd, &input).resolve(&t.state).state;
        assert_eq!(s.dragging, None);
        assert!(ResizableMachine::transition(&s, ResizableEvent::DragTo(50.0), &input).resolve(&s).commands.is_empty());
    }

    #[test]
    fn collapsible_snaps_shut_and_restores() {
        let input = input(vec![panel(20.0, 50.0, true), panel(30.0, 100.0, false)]);
        let s = ResizableState { sizes: vec![30.0, 70.0], ..ResizableMachine::initial(&input) };
        let s = ResizableMachine::transition(&s, ResizableEvent::DragStart(0), &input).resolve(&s).state;
        // Above half the min clamps to min; below it collapses.
        assert!(close(
            &ResizableMachine::transition(&s, ResizableEvent::DragTo(12.0), &input).resolve(&s).state.sizes,
            &[20.0, 80.0]
        ));
        let s = ResizableMachine::transition(&s, ResizableEvent::DragTo(5.0), &input).resolve(&s).state;
        assert!(close(&s.sizes, &[0.0, 100.0]));
        let s = ResizableMachine::transition(&s, ResizableEvent::ToggleCollapse(0), &input).resolve(&s).state;
        assert!(close(&s.sizes, &[30.0, 70.0]));
    }

    #[test]
    fn keyboard_steps_home_end_and_toggle() {
        let input = input(vec![panel(10.0, 100.0, false), panel(20.0, 40.0, true)]);
        let s = ResizableState { sizes: vec![70.0, 30.0], ..ResizableMachine::initial(&input) };
        assert_eq!(controlled_panel(&input.panels, 0), 1);
        let grow = ResizableMachine::transition(&s, ResizableEvent::Step { divider: 0, grow: true }, &input).resolve(&s).state;
        assert!(close(&grow.sizes, &[75.0, 25.0]));
        // Home: panel before to its min, limited by the other's max.
        assert!(close(&ResizableMachine::transition(&s, ResizableEvent::Home(0), &input).resolve(&s).state.sizes, &[60.0, 40.0]));
        // End collapses the collapsible panel after.
        let end = ResizableMachine::transition(&s, ResizableEvent::End(0), &input).resolve(&s).state;
        assert!(close(&end.sizes, &[100.0, 0.0]));
        // Stepping back out opens it at its min, then Enter collapses again.
        let back = ResizableMachine::transition(&end, ResizableEvent::Step { divider: 0, grow: false }, &input).resolve(&end).state;
        assert!(close(&back.sizes, &[80.0, 20.0]));
        let shut = ResizableMachine::transition(&back, ResizableEvent::ToggleCollapse(0), &input).resolve(&back).state;
        assert!(close(&shut.sizes, &[100.0, 0.0]));
        let open = ResizableMachine::transition(&shut, ResizableEvent::ToggleCollapse(0), &input).resolve(&shut).state;
        assert!(close(&open.sizes, &[80.0, 20.0]));
    }

    #[test]
    fn controlled_requests_and_waits_for_sync() {
        let mut input = ResizableInput { sizes: Some(vec![50.0, 50.0]), ..input(vec![PanelSpec::default(); 2]) };
        let s = ResizableMachine::initial(&input);
        let t = ResizableMachine::transition(&s, ResizableEvent::Step { divider: 0, grow: true }, &input).resolve(&s);
        assert!(close(&t.state.sizes, &[50.0, 50.0]));
        assert_eq!(t.commands, vec![ResizableCommand::SizesChange(vec![55.0, 45.0])]);
        input.sizes = Some(vec![55.0, 45.0]);
        assert!(close(&hydroxy_statechart::sync_or_keep::<ResizableMachine>(&t.state, &input).sizes, &[55.0, 45.0]));
    }

    #[test]
    fn bad_divider_is_ignored() {
        let input = input(vec![PanelSpec::default(); 2]);
        let s = ResizableMachine::initial(&input);
        assert!(ResizableMachine::transition(&s, ResizableEvent::Home(1), &input).resolve(&s).commands.is_empty());
    }
}
