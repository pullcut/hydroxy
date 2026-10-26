//! Saved views connector. Anatomy: root, check, default-mark and separator
//! around a [`Menu`](crate::menu) of views and actions, plus a
//! [`Dialog`](crate::dialog) for save-as, rename and delete. Every part carries
//! `data-scope="saved-views"` and its `data-part`. The menu and the dialog
//! keep their own scopes; this module decides what goes in them.

use crate::menu::MenuEntry;

pub use oxide_machines::saved_views::*;

/// The `data-scope` value on every saved views part.
pub const SCOPE: &str = "saved-views";

/// Menu value for a view.
#[must_use]
pub fn view_value(id: &str) -> String {
    format!("view:{id}")
}

/// Menu value: open save-as.
pub const SAVE_AS: &str = "save-as";
/// Menu value: open rename.
pub const RENAME: &str = "rename";
/// Menu value: open delete.
pub const DELETE: &str = "delete";
/// Menu value: make the active view the default, or stop it being the default.
pub const TOGGLE_DEFAULT: &str = "toggle-default";

/// The menu's entries: every view, then the actions. Rename, default and delete
/// are disabled when no view is active.
#[must_use]
pub fn menu_entries(state: &SavedViewsState) -> Vec<MenuEntry> {
    let no_view = state.active_view().is_none();
    let guarded = |value: &str| {
        let entry = MenuEntry::new(value);
        if no_view { entry.disabled() } else { entry }
    };
    let mut entries: Vec<MenuEntry> = state.views.iter().map(|v| MenuEntry::new(view_value(&v.id))).collect();
    entries.push(MenuEntry::new(SAVE_AS));
    entries.extend([guarded(RENAME), guarded(TOGGLE_DEFAULT), guarded(DELETE)]);
    entries
}

/// The machine event for an activated menu value.
#[must_use]
pub fn menu_event(value: &str, state: &SavedViewsState) -> Option<SavedViewsEvent> {
    if let Some(id) = value.strip_prefix("view:") {
        return Some(SavedViewsEvent::Select(id.to_string()));
    }
    match value {
        SAVE_AS => Some(SavedViewsEvent::OpenSaveAs),
        RENAME => Some(SavedViewsEvent::OpenRename),
        DELETE => Some(SavedViewsEvent::OpenDelete),
        TOGGLE_DEFAULT => {
            let active = state.active_view()?;
            if state.default_view.as_deref() == Some(active.id.as_str()) {
                Some(SavedViewsEvent::ClearDefault)
            } else {
                Some(SavedViewsEvent::SetDefault(active.id.clone()))
            }
        }
        _ => None,
    }
}

/// The trigger's text: the active view's name, or `"Views"`.
#[must_use]
pub fn trigger_text(state: &SavedViewsState) -> String {
    state.active_view().map_or_else(|| "Views".to_string(), |v| v.name.clone())
}

/// The label of the default item: `"Set as default"` or `"Remove as default"`.
#[must_use]
pub fn toggle_default_text(state: &SavedViewsState) -> &'static str {
    let is_default = state.active_view().is_some_and(|v| state.default_view.as_deref() == Some(v.id.as_str()));
    if is_default { "Remove as default" } else { "Set as default" }
}

/// The prompt's heading.
#[must_use]
pub fn prompt_title(prompt: &Prompt) -> &'static str {
    match prompt.kind {
        PromptKind::SaveAs => "Save view",
        PromptKind::Rename(_) => "Rename view",
        PromptKind::Delete(_) => "Delete view",
    }
}

/// The confirm button's text.
#[must_use]
pub fn prompt_submit(prompt: &Prompt) -> &'static str {
    match prompt.kind {
        PromptKind::SaveAs => "Save",
        PromptKind::Rename(_) => "Rename",
        PromptKind::Delete(_) => "Delete",
    }
}

/// Supporting text under the heading.
#[must_use]
pub fn prompt_description(prompt: &Prompt, state: &SavedViewsState) -> String {
    match &prompt.kind {
        PromptKind::SaveAs => "Saves the current filters under a name you can come back to.".to_string(),
        PromptKind::Rename(_) => "Pick a new name. The filters stay as they are.".to_string(),
        PromptKind::Delete(id) => {
            let name = state.views.iter().find(|v| v.id == *id).map_or("this view", |v| v.name.as_str());
            format!("Delete \"{name}\"? The filters saved in it are lost.")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state() -> SavedViewsState {
        SavedViewsState {
            views: vec![SavedView::new("a", "Alpha"), SavedView::new("b", "Beta")],
            active: Some("a".into()),
            ..SavedViewsState::default()
        }
    }

    #[test]
    fn entries_list_views_then_actions() {
        let values: Vec<_> = menu_entries(&state()).into_iter().map(|e| (e.value, e.disabled)).collect();
        assert_eq!(values[0].0, "view:a");
        assert_eq!(values[2].0, SAVE_AS);
        assert!(values[3..].iter().all(|(_, disabled)| !disabled));
        let mut none = state();
        none.active = None;
        let disabled: Vec<_> = menu_entries(&none).into_iter().map(|e| e.disabled).collect();
        assert_eq!(disabled, [false, false, false, true, true, true]);
    }

    #[test]
    fn values_map_to_events() {
        let mut s = state();
        assert_eq!(menu_event("view:b", &s), Some(SavedViewsEvent::Select("b".into())));
        assert_eq!(menu_event(SAVE_AS, &s), Some(SavedViewsEvent::OpenSaveAs));
        assert_eq!(menu_event(TOGGLE_DEFAULT, &s), Some(SavedViewsEvent::SetDefault("a".into())));
        assert_eq!(toggle_default_text(&s), "Set as default");
        s.default_view = Some("a".into());
        assert_eq!(menu_event(TOGGLE_DEFAULT, &s), Some(SavedViewsEvent::ClearDefault));
        assert_eq!(toggle_default_text(&s), "Remove as default");
        assert_eq!(menu_event("nope", &s), None);
        assert_eq!(trigger_text(&s), "Alpha");
    }

    #[test]
    fn prompt_text_names_the_view() {
        let p = Prompt { kind: PromptKind::Delete("b".into()), name: String::new(), error: None };
        assert_eq!(prompt_description(&p, &state()), "Delete \"Beta\"? The filters saved in it are lost.");
        assert_eq!(prompt_submit(&p), "Delete");
    }
}
