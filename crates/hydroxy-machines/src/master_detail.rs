//! Master/detail: which pane shows on a narrow screen, and where focus goes on Back.
//!
//! Wide screens render both panes, so only the narrow-screen choice lives here.
//! The host owns the selection; the machine only requests changes and remembers
//! which row to refocus once the host accepts the list view again.

use hydroxy_statechart::{Machine, Transition};

/// A narrow-screen pane.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Pane {
    /// The list.
    #[default]
    List,
    /// The detail.
    Detail,
}

impl Pane {
    /// The `data-mobile-pane` value.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Pane::List => "list",
            Pane::Detail => "detail",
        }
    }
}

/// Which pane the host wants.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum PaneMode {
    /// Follow the selection.
    #[default]
    Auto,
    /// Always the list.
    List,
    /// Always the detail.
    Detail,
}

/// Why the selection changed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reason {
    /// A row was chosen.
    Selection,
    /// Back was pressed.
    Back,
}

/// Props for [`MasterDetailMachine`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MasterDetailInput {
    /// The host's selected row.
    pub selected: Option<String>,
    /// Which pane the host wants on narrow screens.
    pub mode: PaneMode,
}

/// What [`MasterDetailMachine`] remembers between events.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MasterDetailState {
    /// The selection last seen from the host.
    pub selected: Option<String>,
    /// The row to refocus after Back is accepted.
    pub pending_back: Option<String>,
}

impl MasterDetailState {
    /// The pane to show on a narrow screen.
    #[must_use]
    pub fn pane(&self, input: &MasterDetailInput) -> Pane {
        match input.mode {
            PaneMode::List => Pane::List,
            PaneMode::Detail => Pane::Detail,
            PaneMode::Auto => {
                if self.selected.is_some() { Pane::Detail } else { Pane::List }
            }
        }
    }
}

/// Inputs to [`MasterDetailMachine`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MasterDetailEvent {
    /// The host's selection changed.
    Sync,
    /// A row was activated.
    Select(String),
    /// The Back button was pressed.
    Back,
}

/// Side effects from [`MasterDetailMachine`] for the adapter to run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MasterDetailCommand {
    /// Ask the host to change the selection.
    SelectionChange(Option<String>, Reason),
    /// Ask the host to show the list pane.
    PaneChange(Pane),
    /// Back was pressed on this row. The host routes.
    Back(String),
    /// Focus this row (or the list heading when it is gone).
    FocusRow(String),
}

/// Selection requests and Back focus restoration.
#[derive(Debug, Clone, Copy)]
pub struct MasterDetailMachine;

impl Machine for MasterDetailMachine {
    type Input = MasterDetailInput;
    type State = MasterDetailState;
    type Event = MasterDetailEvent;
    type Command = MasterDetailCommand;

    fn initial(input: &MasterDetailInput) -> MasterDetailState {
        MasterDetailState { selected: input.selected.clone(), pending_back: None }
    }

    fn transition(
        state: &MasterDetailState,
        event: MasterDetailEvent,
        input: &MasterDetailInput,
    ) -> Transition<MasterDetailState, MasterDetailCommand> {
        let mut next = state.clone();
        let mut commands = Vec::new();
        match event {
            MasterDetailEvent::Sync => {
                next.selected.clone_from(&input.selected);
                if next.selected.is_some() && next.selected != state.pending_back {
                    next.pending_back = None;
                }
                // The host accepted the list view: hand focus back to the row.
                if next.selected.is_none()
                    && let Some(id) = next.pending_back.take()
                {
                    commands.push(MasterDetailCommand::FocusRow(id));
                }
            }
            MasterDetailEvent::Select(id) => {
                commands.push(MasterDetailCommand::SelectionChange(Some(id), Reason::Selection));
            }
            MasterDetailEvent::Back => {
                if let Some(id) = state.selected.clone() {
                    next.pending_back = Some(id.clone());
                    commands.push(MasterDetailCommand::SelectionChange(None, Reason::Back));
                    if input.mode != PaneMode::Auto {
                        commands.push(MasterDetailCommand::PaneChange(Pane::List));
                    }
                    commands.push(MasterDetailCommand::Back(id));
                }
            }
        }
        Transition::with(next, commands)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use MasterDetailCommand as C;
    use MasterDetailEvent as E;

    fn input(selected: Option<&str>, mode: PaneMode) -> MasterDetailInput {
        MasterDetailInput { selected: selected.map(str::to_string), mode }
    }

    #[test]
    fn auto_pane_follows_selection() {
        let none = input(None, PaneMode::Auto);
        let some = input(Some("a"), PaneMode::Auto);
        assert_eq!(MasterDetailMachine::initial(&none).pane(&none), Pane::List);
        assert_eq!(MasterDetailMachine::initial(&some).pane(&some), Pane::Detail);
        let forced = input(Some("a"), PaneMode::List);
        assert_eq!(MasterDetailMachine::initial(&forced).pane(&forced), Pane::List);
    }

    #[test]
    fn select_only_requests() {
        let i = input(None, PaneMode::Auto);
        let t = { let start = &MasterDetailMachine::initial(&i); MasterDetailMachine::transition(start, E::Select("a".into()), &i).resolve(start) };
        assert_eq!(t.commands, [C::SelectionChange(Some("a".into()), Reason::Selection)]);
        assert_eq!(t.state.selected, None);
    }

    #[test]
    fn back_restores_focus_only_after_the_host_accepts() {
        let i = input(Some("a"), PaneMode::Auto);
        let s = MasterDetailMachine::initial(&i);
        let back = MasterDetailMachine::transition(&s, E::Back, &i).resolve(&s);
        assert_eq!(back.commands, [C::SelectionChange(None, Reason::Back), C::Back("a".into())]);
        // Host refuses: still selected, no focus move.
        let refused = MasterDetailMachine::transition(&back.state, E::Sync, &i).resolve(&back.state);
        assert!(refused.commands.is_empty());
        // Host accepts.
        let accepted_input = input(None, PaneMode::Auto);
        let accepted = MasterDetailMachine::transition(&back.state, E::Sync, &accepted_input).resolve(&back.state);
        assert_eq!(accepted.commands, [C::FocusRow("a".into())]);
        assert_eq!(accepted.state.pending_back, None);
    }

    #[test]
    fn back_in_explicit_mode_asks_for_the_list() {
        let i = input(Some("a"), PaneMode::Detail);
        let t = { let start = &MasterDetailMachine::initial(&i); MasterDetailMachine::transition(start, E::Back, &i).resolve(start) };
        assert!(t.commands.contains(&C::PaneChange(Pane::List)));
    }
}
