//! App shell: the responsive navigation drawer's open state.
//!
//! On wide screens the navigation sits beside the main region and this state is
//! unused. On narrow screens it is a drawer. The machine owns the drawer's open
//! flag and says why it changed; the adapter drives a native popover from it.

use hydroxy_statechart::{Machine, Transition};

/// How the main region scrolls.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ScrollMode {
    /// The page scrolls.
    #[default]
    Document,
    /// Only the main region scrolls, inside a fixed-height shell.
    Main,
}

impl ScrollMode {
    /// The `data-scroll-mode` value.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            ScrollMode::Document => "document",
            ScrollMode::Main => "main",
        }
    }
}

/// Why the drawer opened or closed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavReason {
    /// The menu button.
    Trigger,
    /// Escape, a click outside, a link inside, or the close button.
    Dismiss,
}

/// Props for [`AppShellMachine`].
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct AppShellInput {
    /// Controlled drawer state.
    pub open: Option<bool>,
    /// Initial drawer state when uncontrolled.
    pub default_open: bool,
}

/// What [`AppShellMachine`] remembers between events.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct AppShellState {
    /// The drawer is open.
    pub open: bool,
}

/// Inputs to [`AppShellMachine`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppShellEvent {
    /// Ask to open the drawer.
    Open,
    /// Ask to close the drawer.
    Close,
    /// The popover reported that it opened or closed.
    Toggled(bool),
    /// The controlled value changed.
    Sync,
}

/// Side effects from [`AppShellMachine`] for the adapter to run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppShellCommand {
    /// Tell the host the drawer wants to be open or closed.
    OpenChange(bool, NavReason),
    /// Show or hide the native popover.
    SetPopover(bool),
}

/// The drawer's open flag.
#[derive(Debug, Clone, Copy)]
pub struct AppShellMachine;

impl AppShellMachine {
    /// Reconcile the state with the controlled prop.
    #[must_use]
    pub fn sync(state: &AppShellState, input: &AppShellInput) -> AppShellState {
        AppShellState { open: input.open.unwrap_or(state.open) }
    }
}

impl Machine for AppShellMachine {
    type Input = AppShellInput;
    type State = AppShellState;
    type Event = AppShellEvent;
    type Command = AppShellCommand;

    fn initial(input: &AppShellInput) -> AppShellState {
        AppShellState { open: input.open.unwrap_or(input.default_open) }
    }

    fn transition(
        state: &AppShellState,
        event: AppShellEvent,
        input: &AppShellInput,
    ) -> Transition<AppShellState, AppShellCommand> {
        let controlled = input.open.is_some();
        let request = |open: bool, reason: NavReason| {
            let mut commands = vec![AppShellCommand::OpenChange(open, reason)];
            if !controlled {
                commands.push(AppShellCommand::SetPopover(open));
            }
            Transition::with(AppShellState { open: if controlled { state.open } else { open } }, commands)
        };
        match event {
            AppShellEvent::Open if !state.open => request(true, NavReason::Trigger),
            AppShellEvent::Close if state.open => request(false, NavReason::Dismiss),
            // The browser closed or opened the popover on its own (Escape, light dismiss).
            AppShellEvent::Toggled(open) if open != state.open => {
                let mut commands = vec![AppShellCommand::OpenChange(open, NavReason::Dismiss)];
                if controlled {
                    // Hold the host's value: put the popover back where the host says it is.
                    commands.push(AppShellCommand::SetPopover(state.open));
                    return Transition::emit(commands);
                }
                if open {
                    commands[0] = AppShellCommand::OpenChange(true, NavReason::Trigger);
                }
                Transition::with(AppShellState { open }, commands)
            }
            AppShellEvent::Sync => {
                let synced = Self::sync(state, input);
                let commands =
                    if synced.open == state.open { Vec::new() } else { vec![AppShellCommand::SetPopover(synced.open)] };
                Transition::with(synced, commands)
            }
            _ => Transition::emit(Vec::new()),
        }
    }
}

/// The skip link needs both an id and a label, or neither.
///
/// # Errors
///
/// A message naming the missing half.
pub fn check_skip_link(main_id: Option<&str>, skip_label: Option<&str>) -> Result<bool, &'static str> {
    let has = |s: Option<&str>| s.is_some_and(|v| !v.trim().is_empty());
    match (has(main_id), has(skip_label)) {
        (true, true) => Ok(true),
        (false, false) => Ok(false),
        _ => Err("AppShell needs main_id and skip_label together"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use AppShellCommand as C;

    fn t(
        state: AppShellState,
        event: AppShellEvent,
        input: AppShellInput,
    ) -> hydroxy_statechart::Step<AppShellState, AppShellCommand> {
        AppShellMachine::transition(&state, event, &input).resolve(&state)
    }

    #[test]
    fn uncontrolled_open_and_close() {
        let input = AppShellInput::default();
        let s = AppShellMachine::initial(&input);
        let opened = t(s, AppShellEvent::Open, input);
        assert!(opened.state.open);
        assert_eq!(opened.commands, [C::OpenChange(true, NavReason::Trigger), C::SetPopover(true)]);
        let closed = t(opened.state, AppShellEvent::Close, input);
        assert!(!closed.state.open);
        assert_eq!(closed.commands, [C::OpenChange(false, NavReason::Dismiss), C::SetPopover(false)]);
    }

    #[test]
    fn controlled_host_keeps_authority() {
        let input = AppShellInput { open: Some(false), default_open: false };
        let s = AppShellMachine::initial(&input);
        let asked = t(s, AppShellEvent::Open, input);
        assert!(!asked.state.open);
        assert_eq!(asked.commands, [C::OpenChange(true, NavReason::Trigger)]);
        let synced = t(asked.state, AppShellEvent::Sync, AppShellInput { open: Some(true), default_open: false });
        assert!(synced.state.open);
        assert_eq!(synced.commands, [C::SetPopover(true)]);
    }

    #[test]
    fn escape_is_a_dismiss() {
        let input = AppShellInput::default();
        let r = t(AppShellState { open: true }, AppShellEvent::Toggled(false), input);
        assert!(!r.state.open);
        assert_eq!(r.commands, [C::OpenChange(false, NavReason::Dismiss)]);
        let held = t(
            AppShellState { open: true },
            AppShellEvent::Toggled(false),
            AppShellInput { open: Some(true), default_open: false },
        );
        assert!(held.state.open);
        assert_eq!(held.commands, [C::OpenChange(false, NavReason::Dismiss), C::SetPopover(true)]);
    }

    #[test]
    fn skip_link_is_all_or_nothing() {
        assert_eq!(check_skip_link(Some("main"), Some("Skip to content")), Ok(true));
        assert_eq!(check_skip_link(None, None), Ok(false));
        assert!(check_skip_link(Some("main"), None).is_err());
    }
}
