//! Saved views owns a list of named presets, which one is active, which one is
//! the default, and the save-as, rename and delete prompts. It stores no
//! filters: a view is an id and a name, and the owner maps ids to its own
//! settings when it hears [`SavedViewsCommand::Created`] or `ActiveChange`.

use hydroxy_statechart::{Machine, Transition};

/// One preset.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SavedView {
    /// Unique key. The owner keys its own settings on this.
    pub id: String,
    /// Shown in the menu.
    pub name: String,
}

impl SavedView {
    /// A view.
    #[must_use]
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self { id: id.into(), name: name.into() }
    }
}

/// Props for [`SavedViewsMachine`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SavedViewsInput {
    /// Controlled views. `Some` at start fixes controlled mode.
    pub views: Option<Vec<SavedView>>,
    /// Uncontrolled initial views.
    pub default_views: Vec<SavedView>,
    /// Controlled active view. `Some` at start fixes controlled mode.
    pub active: Option<String>,
    /// Uncontrolled initial active view.
    pub default_active: Option<String>,
    /// Initial default view: the one to open with. Changes are reported, not controlled.
    pub default_view: Option<String>,
}

/// What the prompt is for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PromptKind {
    /// Name a new view.
    SaveAs,
    /// Rename this view.
    Rename(String),
    /// Confirm deleting this view.
    Delete(String),
}

/// Why a name was turned down.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NameError {
    /// Nothing but spaces.
    Empty,
    /// Another view already has it.
    Taken,
}

impl NameError {
    /// A short sentence for the user.
    #[must_use]
    pub const fn message(self) -> &'static str {
        match self {
            NameError::Empty => "Give the view a name",
            NameError::Taken => "A view with that name already exists",
        }
    }
}

impl std::fmt::Display for NameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.message())
    }
}

impl std::error::Error for NameError {}

/// An open prompt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Prompt {
    /// What it's for.
    pub kind: PromptKind,
    /// The name being typed (save-as and rename).
    pub name: String,
    /// Why the last submit failed.
    pub error: Option<NameError>,
}

/// What [`SavedViewsMachine`] remembers between events.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SavedViewsState {
    /// The presets.
    pub views: Vec<SavedView>,
    /// The view in use.
    pub active: Option<String>,
    /// The view to open with.
    pub default_view: Option<String>,
    /// The open prompt, if any.
    pub prompt: Option<Prompt>,
    /// Fixed at start: the owner holds the views.
    pub controlled_views: bool,
    /// Fixed at start: the owner holds the active view.
    pub controlled_active: bool,
    /// Counter behind new ids.
    pub counter: u32,
}

impl SavedViewsState {
    /// The active view, if it still exists.
    #[must_use]
    pub fn active_view(&self) -> Option<&SavedView> {
        let id = self.active.as_deref()?;
        self.views.iter().find(|v| v.id == id)
    }
}

/// Inputs to [`SavedViewsMachine`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SavedViewsEvent {
    /// Use this view.
    Select(String),
    /// Use no view.
    Deselect,
    /// Open the save-as prompt.
    OpenSaveAs,
    /// Open the rename prompt for the active view.
    OpenRename,
    /// Open the delete prompt for the active view.
    OpenDelete,
    /// The name field changed.
    EditName(String),
    /// Confirm the open prompt.
    Submit,
    /// Dismiss the open prompt.
    ClosePrompt,
    /// Make this view the one to open with.
    SetDefault(String),
    /// Stop having a default view.
    ClearDefault,
}

/// Side effects from [`SavedViewsMachine`] for the adapter to run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SavedViewsCommand {
    /// Save-as made this view. Store the owner's current settings under its id.
    Created(SavedView),
    /// A view was renamed.
    Renamed {
        /// The view.
        id: String,
        /// Its new name.
        name: String,
    },
    /// A view was deleted.
    Deleted(String),
    /// The active view changed. `None` means no view.
    ActiveChange(Option<String>),
    /// The default view changed. `None` means no default.
    DefaultChange(Option<String>),
}

/// Apply a command to a view list. Controlled owners call this from their handler.
pub fn apply(views: &mut Vec<SavedView>, command: &SavedViewsCommand) {
    match command {
        SavedViewsCommand::Created(view) => views.push(view.clone()),
        SavedViewsCommand::Renamed { id, name } => {
            if let Some(v) = views.iter_mut().find(|v| v.id == *id) {
                v.name.clone_from(name);
            }
        }
        SavedViewsCommand::Deleted(id) => views.retain(|v| v.id != *id),
        SavedViewsCommand::ActiveChange(_) | SavedViewsCommand::DefaultChange(_) => {}
    }
}

/// Check a name against the other views. Returns the trimmed name.
///
/// # Errors
/// [`NameError::Empty`] for a blank name, [`NameError::Taken`] when another view
/// (not `except`) has the same name, ignoring case.
pub fn validate_name(views: &[SavedView], name: &str, except: Option<&str>) -> Result<String, NameError> {
    let name = name.trim();
    if name.is_empty() {
        return Err(NameError::Empty);
    }
    let taken = views.iter().any(|v| Some(v.id.as_str()) != except && v.name.trim().eq_ignore_ascii_case(name));
    if taken { Err(NameError::Taken) } else { Ok(name.to_string()) }
}

/// Presets, the active one and the prompts around them.
#[derive(Debug, Clone, Copy)]
pub struct SavedViewsMachine;

type Step = Transition<SavedViewsState, SavedViewsCommand>;

fn fresh_id(views: &[SavedView], counter: &mut u32) -> String {
    loop {
        *counter += 1;
        let id = format!("view-{counter}");
        if views.iter().all(|v| v.id != id) {
            return id;
        }
    }
}

impl Machine for SavedViewsMachine {
    type Input = SavedViewsInput;
    type State = SavedViewsState;
    type Event = SavedViewsEvent;
    type Command = SavedViewsCommand;

    fn initial(input: &SavedViewsInput) -> SavedViewsState {
        let views = input.views.clone().unwrap_or_else(|| input.default_views.clone());
        let exists = |id: &Option<String>| id.as_ref().filter(|id| views.iter().any(|v| v.id == **id)).cloned();
        let active = exists(&input.active.clone().or_else(|| input.default_active.clone()));
        let default_view = exists(&input.default_view);
        SavedViewsState {
            views,
            active,
            default_view,
            prompt: None,
            controlled_views: input.views.is_some(),
            controlled_active: input.active.is_some(),
            counter: 0,
        }
    }

    #[allow(clippy::too_many_lines, reason = "one match over the events; splitting it only adds plumbing")]
    fn transition(state: &SavedViewsState, event: SavedViewsEvent, _input: &SavedViewsInput) -> Step {
        let stay = || Transition::stay();
        let mut next = state.clone();
        let mut commands = Vec::new();
        let has = |id: &str| state.views.iter().any(|v| v.id == id);
        match event {
            SavedViewsEvent::Select(id) => {
                if !has(&id) || state.active.as_deref() == Some(id.as_str()) {
                    return stay();
                }
                if !state.controlled_active {
                    next.active = Some(id.clone());
                }
                commands.push(SavedViewsCommand::ActiveChange(Some(id)));
            }
            SavedViewsEvent::Deselect => {
                if state.active.is_none() {
                    return stay();
                }
                if !state.controlled_active {
                    next.active = None;
                }
                commands.push(SavedViewsCommand::ActiveChange(None));
            }
            SavedViewsEvent::OpenSaveAs => {
                next.prompt = Some(Prompt { kind: PromptKind::SaveAs, name: String::new(), error: None });
            }
            SavedViewsEvent::OpenRename => {
                let Some(view) = state.active_view() else { return stay() };
                next.prompt =
                    Some(Prompt { kind: PromptKind::Rename(view.id.clone()), name: view.name.clone(), error: None });
            }
            SavedViewsEvent::OpenDelete => {
                let Some(view) = state.active_view() else { return stay() };
                next.prompt =
                    Some(Prompt { kind: PromptKind::Delete(view.id.clone()), name: String::new(), error: None });
            }
            SavedViewsEvent::EditName(name) => {
                let Some(prompt) = next.prompt.as_mut().filter(|p| !matches!(p.kind, PromptKind::Delete(_))) else {
                    return stay();
                };
                prompt.name = name;
                prompt.error = None;
            }
            SavedViewsEvent::ClosePrompt => {
                if state.prompt.is_none() {
                    return stay();
                }
                next.prompt = None;
            }
            SavedViewsEvent::Submit => {
                let Some(prompt) = state.prompt.clone() else { return stay() };
                match prompt.kind.clone() {
                    PromptKind::SaveAs => match validate_name(&state.views, &prompt.name, None) {
                        Err(error) => {
                            next.prompt = Some(Prompt { error: Some(error), ..prompt });
                            return Transition::to(next);
                        }
                        Ok(name) => {
                            let view = SavedView::new(fresh_id(&state.views, &mut next.counter), name);
                            if !state.controlled_views {
                                next.views.push(view.clone());
                            }
                            if !state.controlled_active {
                                next.active = Some(view.id.clone());
                            }
                            commands.push(SavedViewsCommand::ActiveChange(Some(view.id.clone())));
                            commands.insert(0, SavedViewsCommand::Created(view));
                        }
                    },
                    PromptKind::Rename(id) => match validate_name(&state.views, &prompt.name, Some(&id)) {
                        Err(error) => {
                            next.prompt = Some(Prompt { error: Some(error), ..prompt });
                            return Transition::to(next);
                        }
                        Ok(name) => {
                            let same = state.views.iter().any(|v| v.id == id && v.name == name);
                            if has(&id) && !same {
                                apply_to(
                                    &mut next,
                                    state.controlled_views,
                                    &SavedViewsCommand::Renamed { id: id.clone(), name: name.clone() },
                                );
                                commands.push(SavedViewsCommand::Renamed { id, name });
                            }
                        }
                    },
                    PromptKind::Delete(id) => {
                        if has(&id) {
                            apply_to(&mut next, state.controlled_views, &SavedViewsCommand::Deleted(id.clone()));
                            commands.push(SavedViewsCommand::Deleted(id.clone()));
                            if state.active.as_deref() == Some(id.as_str()) {
                                if !state.controlled_active {
                                    next.active = None;
                                }
                                commands.push(SavedViewsCommand::ActiveChange(None));
                            }
                            if state.default_view.as_deref() == Some(id.as_str()) {
                                next.default_view = None;
                                commands.push(SavedViewsCommand::DefaultChange(None));
                            }
                        }
                    }
                }
                next.prompt = None;
            }
            SavedViewsEvent::SetDefault(id) => {
                if !has(&id) || state.default_view.as_deref() == Some(id.as_str()) {
                    return stay();
                }
                next.default_view = Some(id.clone());
                commands.push(SavedViewsCommand::DefaultChange(Some(id)));
            }
            SavedViewsEvent::ClearDefault => {
                if state.default_view.is_none() {
                    return stay();
                }
                next.default_view = None;
                commands.push(SavedViewsCommand::DefaultChange(None));
            }
        }
        Transition::with(next, commands)
    }

    fn sync(state: &SavedViewsState, input: &SavedViewsInput) -> Option<SavedViewsState> {
        let mut next = state.clone();
        if state.controlled_views {
            next.views = input.views.clone().unwrap_or_default();
        }
        if state.controlled_active {
            next.active.clone_from(&input.active);
        }
        let has =
            |views: &[SavedView], id: &Option<String>| id.as_ref().is_some_and(|id| views.iter().any(|v| v.id == *id));
        if !has(&next.views, &next.active) && !state.controlled_active {
            next.active = None;
        }
        if !has(&next.views, &next.default_view) {
            next.default_view = None;
        }
        let target = next.prompt.as_ref().and_then(|p| match &p.kind {
            PromptKind::SaveAs => None,
            PromptKind::Rename(id) | PromptKind::Delete(id) => Some(id.clone()),
        });
        if target.is_some_and(|id| next.views.iter().all(|v| v.id != id)) {
            next.prompt = None;
        }
        hydroxy_statechart::changed(state, next)
    }
}

/// Apply a view-list command to the state when the machine owns the list.
fn apply_to(state: &mut SavedViewsState, controlled: bool, command: &SavedViewsCommand) {
    if !controlled {
        apply(&mut state.views, command);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hydroxy_statechart::step;

    fn views() -> Vec<SavedView> {
        vec![SavedView::new("mine", "My open tickets"), SavedView::new("urgent", "Urgent")]
    }

    fn input() -> SavedViewsInput {
        SavedViewsInput { default_views: views(), default_active: Some("mine".into()), ..SavedViewsInput::default() }
    }

    fn names(s: &SavedViewsState) -> Vec<&str> {
        s.views.iter().map(|v| v.name.as_str()).collect()
    }

    #[test]
    fn select_changes_the_active_view_once() {
        let i = input();
        let s = SavedViewsMachine::initial(&i);
        assert_eq!(s.active.as_deref(), Some("mine"));
        let step = hydroxy_statechart::step::<SavedViewsMachine>(&s, SavedViewsEvent::Select("urgent".into()), &i);
        assert_eq!(step.state.active.as_deref(), Some("urgent"));
        assert_eq!(step.commands, [SavedViewsCommand::ActiveChange(Some("urgent".into()))]);
        assert!(
            hydroxy_statechart::step::<SavedViewsMachine>(&step.state, SavedViewsEvent::Select("urgent".into()), &i)
                .commands
                .is_empty()
        );
        assert!(
            hydroxy_statechart::step::<SavedViewsMachine>(&s, SavedViewsEvent::Select("nope".into()), &i)
                .commands
                .is_empty()
        );
        let step = hydroxy_statechart::step::<SavedViewsMachine>(&s, SavedViewsEvent::Deselect, &i);
        assert_eq!((step.state.active, step.commands), (None, vec![SavedViewsCommand::ActiveChange(None)]));
    }

    #[test]
    fn save_as_validates_then_creates_and_activates() {
        let i = input();
        let s = hydroxy_statechart::step::<SavedViewsMachine>(
            &SavedViewsMachine::initial(&i),
            SavedViewsEvent::OpenSaveAs,
            &i,
        )
        .state;
        let blank = hydroxy_statechart::step::<SavedViewsMachine>(&s, SavedViewsEvent::Submit, &i).state;
        assert_eq!(blank.prompt.as_ref().and_then(|p| p.error), Some(NameError::Empty));
        let s =
            hydroxy_statechart::step::<SavedViewsMachine>(&blank, SavedViewsEvent::EditName("  urgent ".into()), &i)
                .state;
        assert_eq!(s.prompt.as_ref().and_then(|p| p.error), None);
        let taken = hydroxy_statechart::step::<SavedViewsMachine>(&s, SavedViewsEvent::Submit, &i).state;
        assert_eq!(taken.prompt.as_ref().and_then(|p| p.error), Some(NameError::Taken));
        let s = hydroxy_statechart::step::<SavedViewsMachine>(
            &taken,
            SavedViewsEvent::EditName(" Waiting on me ".into()),
            &i,
        )
        .state;
        let step = hydroxy_statechart::step::<SavedViewsMachine>(&s, SavedViewsEvent::Submit, &i);
        assert_eq!(names(&step.state), ["My open tickets", "Urgent", "Waiting on me"]);
        assert_eq!(step.state.active.as_deref(), Some("view-1"));
        assert!(step.state.prompt.is_none());
        assert_eq!(
            step.commands,
            [
                SavedViewsCommand::Created(SavedView::new("view-1", "Waiting on me")),
                SavedViewsCommand::ActiveChange(Some("view-1".into())),
            ]
        );
    }

    #[test]
    fn rename_keeps_its_own_name_legal_and_only_reports_changes() {
        let i = input();
        let s = hydroxy_statechart::step::<SavedViewsMachine>(
            &SavedViewsMachine::initial(&i),
            SavedViewsEvent::OpenRename,
            &i,
        )
        .state;
        assert_eq!(s.prompt.as_ref().map(|p| p.name.as_str()), Some("My open tickets"));
        // Submitting the unchanged name just closes.
        let step = hydroxy_statechart::step::<SavedViewsMachine>(&s, SavedViewsEvent::Submit, &i);
        assert!(step.commands.is_empty() && step.state.prompt.is_none());
        let s = hydroxy_statechart::step::<SavedViewsMachine>(&s, SavedViewsEvent::EditName("Mine".into()), &i).state;
        let step = hydroxy_statechart::step::<SavedViewsMachine>(&s, SavedViewsEvent::Submit, &i);
        assert_eq!(names(&step.state), ["Mine", "Urgent"]);
        assert_eq!(step.commands, [SavedViewsCommand::Renamed { id: "mine".into(), name: "Mine".into() }]);
        // Another view's name is taken.
        let s = hydroxy_statechart::step::<SavedViewsMachine>(
            &SavedViewsMachine::initial(&i),
            SavedViewsEvent::OpenRename,
            &i,
        )
        .state;
        let s = hydroxy_statechart::step::<SavedViewsMachine>(&s, SavedViewsEvent::EditName("URGENT".into()), &i).state;
        assert_eq!(
            hydroxy_statechart::step::<SavedViewsMachine>(&s, SavedViewsEvent::Submit, &i)
                .state
                .prompt
                .and_then(|p| p.error),
            Some(NameError::Taken)
        );
    }

    #[test]
    fn delete_clears_active_and_default() {
        let i = SavedViewsInput { default_view: Some("mine".into()), ..input() };
        let s = SavedViewsMachine::initial(&i);
        assert_eq!(s.default_view.as_deref(), Some("mine"));
        let s = hydroxy_statechart::step::<SavedViewsMachine>(&s, SavedViewsEvent::OpenDelete, &i).state;
        let step = hydroxy_statechart::step::<SavedViewsMachine>(&s, SavedViewsEvent::Submit, &i);
        assert_eq!(names(&step.state), ["Urgent"]);
        assert_eq!((step.state.active.as_deref(), step.state.default_view.as_deref()), (None, None));
        assert_eq!(
            step.commands,
            [
                SavedViewsCommand::Deleted("mine".into()),
                SavedViewsCommand::ActiveChange(None),
                SavedViewsCommand::DefaultChange(None),
            ]
        );
        // Nothing active: no rename or delete prompt.
        assert!(
            hydroxy_statechart::step::<SavedViewsMachine>(&step.state, SavedViewsEvent::OpenDelete, &i)
                .state
                .prompt
                .is_none()
        );
    }

    #[test]
    fn default_view_can_be_set_and_cleared() {
        let i = input();
        let s = SavedViewsMachine::initial(&i);
        let step = hydroxy_statechart::step::<SavedViewsMachine>(&s, SavedViewsEvent::SetDefault("urgent".into()), &i);
        assert_eq!(step.commands, [SavedViewsCommand::DefaultChange(Some("urgent".into()))]);
        assert!(
            hydroxy_statechart::step::<SavedViewsMachine>(
                &step.state,
                SavedViewsEvent::SetDefault("urgent".into()),
                &i
            )
            .commands
            .is_empty()
        );
        let step = hydroxy_statechart::step::<SavedViewsMachine>(&step.state, SavedViewsEvent::ClearDefault, &i);
        assert_eq!(step.state.default_view, None);
    }

    #[test]
    fn controlled_views_and_active_wait_for_the_owner() {
        let mut i = SavedViewsInput { views: Some(views()), active: Some("mine".into()), ..SavedViewsInput::default() };
        let s = SavedViewsMachine::initial(&i);
        let step = hydroxy_statechart::step::<SavedViewsMachine>(&s, SavedViewsEvent::Select("urgent".into()), &i);
        assert_eq!(step.state.active.as_deref(), Some("mine"));
        assert_eq!(step.commands.len(), 1);
        let s = hydroxy_statechart::step::<SavedViewsMachine>(&s, SavedViewsEvent::OpenSaveAs, &i).state;
        let s = hydroxy_statechart::step::<SavedViewsMachine>(&s, SavedViewsEvent::EditName("New".into()), &i).state;
        let step = hydroxy_statechart::step::<SavedViewsMachine>(&s, SavedViewsEvent::Submit, &i);
        assert_eq!(names(&step.state), ["My open tickets", "Urgent"]);
        // The owner applies the command and passes the new props back.
        let mut list = views();
        for c in &step.commands {
            apply(&mut list, c);
        }
        i.views = Some(list);
        i.active = Some("view-1".into());
        let synced = hydroxy_statechart::sync_or_keep::<SavedViewsMachine>(&step.state, &i);
        assert_eq!(names(&synced), ["My open tickets", "Urgent", "New"]);
        assert_eq!(synced.active.as_deref(), Some("view-1"));
    }

    #[test]
    fn sync_drops_a_prompt_whose_view_vanished() {
        let mut i = input();
        i.views = Some(views());
        let s = SavedViewsMachine::initial(&i);
        let s = step::<SavedViewsMachine>(&s, SavedViewsEvent::OpenRename, &i).state;
        i.views = Some(vec![SavedView::new("urgent", "Urgent")]);
        let synced = hydroxy_statechart::sync_or_keep::<SavedViewsMachine>(&s, &i);
        assert!(synced.prompt.is_none());
    }

    #[test]
    fn new_ids_skip_taken_ones() {
        let i = SavedViewsInput { default_views: vec![SavedView::new("view-1", "A")], ..SavedViewsInput::default() };
        let s = hydroxy_statechart::step::<SavedViewsMachine>(
            &SavedViewsMachine::initial(&i),
            SavedViewsEvent::OpenSaveAs,
            &i,
        )
        .state;
        let s = hydroxy_statechart::step::<SavedViewsMachine>(&s, SavedViewsEvent::EditName("B".into()), &i).state;
        let step = hydroxy_statechart::step::<SavedViewsMachine>(&s, SavedViewsEvent::Submit, &i);
        assert_eq!(step.state.views[1].id, "view-2");
    }
}
