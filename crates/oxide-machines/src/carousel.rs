//! Carousel: one slide index with clamp-or-wrap navigation, and optional
//! autoplay. Scrolling and snapping are the browser's job (CSS scroll-snap);
//! the machine only says which slide is current.
//!
//! Autoplay pauses while the pointer is over the carousel or focus is inside
//! it, and the owner can pause it with [`CarouselEvent::TogglePlay`]. The
//! machine owns the delay but not the clock: it asks the adapter to
//! [`CarouselCommand::Wait`] and acts on the matching [`CarouselEvent::Elapsed`].
//! A newer wait, or anything that pauses autoplay, makes an older one stale.

use oxide_statechart::{Machine, Transition};

use crate::ChangeReason;

/// Props for [`CarouselMachine`].
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CarouselInput {
    /// Controlled index. `Some` at start fixes controlled mode.
    pub index: Option<usize>,
    /// Uncontrolled initial index.
    pub default_index: usize,
    /// Number of slides.
    pub count: usize,
    /// Wrap past the ends instead of stopping.
    pub looping: bool,
    /// Advance every this many milliseconds. `None` is no autoplay. Autoplay always wraps.
    pub autoplay_ms: Option<u32>,
}

/// What [`CarouselMachine`] remembers between events.
#[allow(clippy::struct_excessive_bools, reason = "independent pause reasons")]
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CarouselState {
    /// The current slide.
    pub index: usize,
    /// Fixed at start: the owner holds `index`.
    pub controlled: bool,
    /// The adapter has said autoplay may begin.
    pub started: bool,
    /// The owner has not paused it.
    pub playing: bool,
    /// The pointer is over the carousel.
    pub hovered: bool,
    /// Focus is inside the carousel.
    pub focused: bool,
    /// The wait in flight, if any.
    pub pending: Option<u32>,
    /// Last wait id handed out.
    pub seq: u32,
}

impl CarouselState {
    /// Whether the autoplay timer is running right now.
    #[must_use]
    pub fn running(&self) -> bool {
        self.pending.is_some()
    }
}

/// Inputs to [`CarouselMachine`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CarouselEvent {
    /// Next slide.
    Next(ChangeReason),
    /// Previous slide.
    Prev(ChangeReason),
    /// A specific slide, such as a dot or a finished swipe.
    Go(usize, ChangeReason),
    /// The pointer entered (`true`) or left the carousel.
    Hover(bool),
    /// Focus entered (`true`) or left the carousel.
    Focus(bool),
    /// The pause / play button.
    TogglePlay,
    /// The adapter is ready (and the user has not asked for reduced motion): begin autoplay.
    Start,
    /// The wait with this id is over.
    Elapsed(u32),
}

/// Side effects from [`CarouselMachine`] for the adapter to run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CarouselCommand {
    /// The index changed (or should, when controlled).
    IndexChange {
        /// The new slide.
        index: usize,
        /// Why.
        reason: ChangeReason,
    },
    /// Send [`CarouselEvent::Elapsed`] with `id` after `ms` milliseconds.
    Wait {
        /// Echo this back in `Elapsed`.
        id: u32,
        /// How long to wait.
        ms: u32,
    },
}

/// Slide index and autoplay for one carousel.
#[derive(Debug, Clone, Copy)]
pub struct CarouselMachine;

type Step = Transition<CarouselState, CarouselCommand>;

fn clamp(index: usize, count: usize) -> usize {
    index.min(count.saturating_sub(1))
}

/// The index `delta` slides away, wrapping or stopping at the ends.
#[must_use]
pub fn step_index(index: usize, delta: isize, count: usize, looping: bool) -> usize {
    if count == 0 {
        return 0;
    }
    let to = i64::try_from(index).unwrap_or(0) + i64::try_from(delta).unwrap_or(0);
    let n = i64::try_from(count).unwrap_or(1);
    let to = if looping { to.rem_euclid(n) } else { to.clamp(0, n - 1) };
    usize::try_from(to).unwrap_or(0)
}

/// Start, replace or drop the autoplay wait so it matches the state.
fn rearm(mut state: CarouselState, input: &CarouselInput, mut commands: Vec<CarouselCommand>) -> Step {
    let ms = input.autoplay_ms.filter(|_| input.count > 1);
    match ms {
        Some(ms) if state.started && state.playing && !state.hovered && !state.focused => {
            state.seq = state.seq.wrapping_add(1);
            state.pending = Some(state.seq);
            commands.push(CarouselCommand::Wait { id: state.seq, ms });
        }
        _ => state.pending = None,
    }
    Transition { state, commands }
}

/// Move to `to`, requesting the change if it is one, then restart autoplay.
fn go(state: &CarouselState, input: &CarouselInput, to: usize, reason: ChangeReason) -> Step {
    let to = clamp(to, input.count);
    let mut next = state.clone();
    let mut commands = Vec::new();
    if to != state.index {
        if !state.controlled {
            next.index = to;
        }
        commands.push(CarouselCommand::IndexChange { index: to, reason });
    }
    // A manual move restarts the countdown, so a slide never gets cut short.
    if state.pending.is_some() { rearm(next, input, commands) } else { Transition { state: next, commands } }
}

impl Machine for CarouselMachine {
    type Input = CarouselInput;
    type State = CarouselState;
    type Event = CarouselEvent;
    type Command = CarouselCommand;

    fn initial(input: &CarouselInput) -> CarouselState {
        CarouselState {
            index: clamp(input.index.unwrap_or(input.default_index), input.count),
            controlled: input.index.is_some(),
            playing: true,
            ..CarouselState::default()
        }
    }

    fn transition(state: &CarouselState, event: CarouselEvent, input: &CarouselInput) -> Step {
        let by = |delta, reason| go(state, input, step_index(state.index, delta, input.count, input.looping), reason);
        match event {
            CarouselEvent::Next(reason) => by(1, reason),
            CarouselEvent::Prev(reason) => by(-1, reason),
            CarouselEvent::Go(index, reason) => go(state, input, index, reason),
            CarouselEvent::Hover(hovered) => rearm(CarouselState { hovered, ..state.clone() }, input, Vec::new()),
            CarouselEvent::Focus(focused) => rearm(CarouselState { focused, ..state.clone() }, input, Vec::new()),
            CarouselEvent::TogglePlay => rearm(CarouselState { playing: !state.playing, ..state.clone() }, input, Vec::new()),
            CarouselEvent::Start if !state.started => rearm(CarouselState { started: true, ..state.clone() }, input, Vec::new()),
            CarouselEvent::Elapsed(id) if state.pending == Some(id) => {
                let to = step_index(state.index, 1, input.count, true);
                let mut commands = Vec::new();
                let mut next = state.clone();
                if to != state.index {
                    if !state.controlled {
                        next.index = to;
                    }
                    commands.push(CarouselCommand::IndexChange { index: to, reason: ChangeReason::Programmatic });
                }
                rearm(next, input, commands)
            }
            _ => Transition::stay(state.clone()),
        }
    }

    fn sync(state: &CarouselState, input: &CarouselInput) -> CarouselState {
        let mut next = state.clone();
        if let (true, Some(index)) = (state.controlled, input.index) {
            next.index = index;
        }
        next.index = clamp(next.index, input.count);
        if input.autoplay_ms.is_none() || input.count < 2 {
            next.pending = None;
        }
        next
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input() -> CarouselInput {
        CarouselInput { count: 3, ..CarouselInput::default() }
    }

    fn run(state: &CarouselState, event: CarouselEvent, input: &CarouselInput) -> Step {
        CarouselMachine::transition(state, event, input)
    }

    const KEY: ChangeReason = ChangeReason::Keyboard;

    #[test]
    fn clamps_or_wraps_at_the_ends() {
        let i = input();
        let s = CarouselMachine::initial(&i);
        assert!(run(&s, CarouselEvent::Prev(KEY), &i).commands.is_empty());
        let last = run(&s, CarouselEvent::Go(99, KEY), &i);
        assert_eq!(last.state.index, 2);
        assert!(run(&last.state, CarouselEvent::Next(KEY), &i).commands.is_empty());
        let looping = CarouselInput { looping: true, ..i.clone() };
        assert_eq!(run(&last.state, CarouselEvent::Next(KEY), &looping).state.index, 0);
        assert_eq!(run(&s, CarouselEvent::Prev(KEY), &looping).state.index, 2);
    }

    #[test]
    fn controlled_index_waits_for_the_owner() {
        let i = CarouselInput { index: Some(1), ..input() };
        let s = CarouselMachine::initial(&i);
        let step = run(&s, CarouselEvent::Next(ChangeReason::Pointer), &i);
        assert_eq!(step.state.index, 1);
        assert_eq!(step.commands, [CarouselCommand::IndexChange { index: 2, reason: ChangeReason::Pointer }]);
        let i2 = CarouselInput { index: Some(2), ..input() };
        assert_eq!(CarouselMachine::sync(&step.state, &i2).index, 2);
    }

    #[test]
    fn sync_clamps_when_slides_go_away() {
        let s = CarouselState { index: 2, ..CarouselState::default() };
        assert_eq!(CarouselMachine::sync(&s, &CarouselInput { count: 2, ..input() }).index, 1);
    }

    fn auto() -> CarouselInput {
        CarouselInput { autoplay_ms: Some(1000), ..input() }
    }

    #[test]
    fn autoplay_waits_advances_and_wraps() {
        let i = auto();
        let s = CarouselMachine::initial(&i);
        assert!(s.pending.is_none(), "idle until started");
        let started = run(&s, CarouselEvent::Start, &i);
        assert_eq!(started.commands, [CarouselCommand::Wait { id: 1, ms: 1000 }]);
        let tick = run(&started.state, CarouselEvent::Elapsed(1), &i);
        assert_eq!(tick.state.index, 1);
        assert_eq!(tick.commands[1], CarouselCommand::Wait { id: 2, ms: 1000 });
        let mut s = tick.state;
        for id in 2..=3 {
            s = run(&s, CarouselEvent::Elapsed(id), &i).state;
        }
        assert_eq!(s.index, 0, "autoplay wraps");
        assert!(run(&s, CarouselEvent::Elapsed(1), &i).commands.is_empty(), "stale waits are ignored");
    }

    #[test]
    fn hover_focus_and_pause_stop_autoplay() {
        let i = auto();
        let s = run(&CarouselMachine::initial(&i), CarouselEvent::Start, &i).state;
        let hovered = run(&s, CarouselEvent::Hover(true), &i).state;
        assert!(!hovered.running());
        assert!(run(&hovered, CarouselEvent::Elapsed(1), &i).commands.is_empty());
        let left = run(&hovered, CarouselEvent::Hover(false), &i);
        assert!(left.state.running());
        let focused = run(&left.state, CarouselEvent::Focus(true), &i).state;
        assert!(!focused.running());
        let paused = run(&run(&focused, CarouselEvent::Focus(false), &i).state, CarouselEvent::TogglePlay, &i).state;
        assert!(!paused.running());
        assert!(run(&paused, CarouselEvent::TogglePlay, &i).state.running());
    }

    #[test]
    fn manual_navigation_restarts_the_countdown() {
        let i = auto();
        let s = run(&CarouselMachine::initial(&i), CarouselEvent::Start, &i).state;
        let step = run(&s, CarouselEvent::Next(ChangeReason::Pointer), &i);
        assert!(step.commands.contains(&CarouselCommand::Wait { id: 2, ms: 1000 }));
        assert!(run(&step.state, CarouselEvent::Elapsed(1), &i).commands.is_empty());
    }

    #[test]
    fn a_single_slide_never_autoplays() {
        let i = CarouselInput { count: 1, ..auto() };
        let s = run(&CarouselMachine::initial(&i), CarouselEvent::Start, &i);
        assert!(s.commands.is_empty());
    }
}
