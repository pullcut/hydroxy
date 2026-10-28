//! `SavedViews`: a menu of saved filter presets with save-as, rename,
//! delete and set-as-default. Built from `Menu` and `Dialog`. It keeps
//! names and ids only; the owner stores the filters themselves.

use dioxus::prelude::*;
use oxide_headless::saved_views::{
    self as h, PromptKind, SavedViewsEvent, SavedViewsInput, SavedViewsMachine, SavedViewsState,
};
use oxide_statechart::Machine;

use crate::button::{Button, ButtonType, ButtonVariant};
use crate::dialog::{Dialog, DialogActions, DialogClose, DialogContent, DialogDescription, DialogTitle};
use crate::icon::{Icon, IconName};
use crate::input::Input;
use crate::menu::{Menu, MenuContent, MenuItem, MenuTrigger};

pub use oxide_headless::saved_views::{SavedView, SavedViewsCommand, apply};

#[derive(Clone, Copy)]
struct Ctx {
    state: Signal<SavedViewsState>,
    input: Memo<SavedViewsInput>,
    on_change: Option<EventHandler<SavedViewsCommand>>,
}

impl Ctx {
    fn send(mut self, event: SavedViewsEvent) {
        let step = SavedViewsMachine::transition(&self.state.peek(), event, &self.input.peek());
        if let Some(next) = step.state {
            self.state.set(next);
        }
        if let Some(handler) = self.on_change {
            for command in step.commands {
                handler.call(command);
            }
        }
    }
}

/// A dropdown of saved views. Every change is reported through `on_change`.
#[allow(clippy::too_many_lines, reason = "one rsx! tree; splitting it only adds prop plumbing")]
#[component]
pub fn SavedViews(
    /// Controlled views. Pair with `on_change` and [`apply`].
    #[props(default)]
    views: ReadSignal<Option<Vec<SavedView>>>,
    /// Initial views when uncontrolled.
    #[props(default)]
    default_views: Vec<SavedView>,
    /// Controlled active view id. Pair with `on_change`.
    #[props(default)]
    active: ReadSignal<Option<String>>,
    /// Initially active view id when uncontrolled.
    default_active: Option<String>,
    /// The view to open with, such as one the owner loaded from storage.
    default_view: Option<String>,
    /// Called with every change: a view created, renamed or deleted, a new active view, a new default.
    on_change: Option<EventHandler<SavedViewsCommand>>,
) -> Element {
    let input = use_memo(move || SavedViewsInput {
        views: views(),
        default_views: default_views.clone(),
        active: active(),
        default_active: default_active.clone(),
        default_view: default_view.clone(),
    });
    let mut state = use_signal(|| SavedViewsMachine::initial(&input.peek()));
    use_effect(move || {
        let synced = SavedViewsMachine::sync(&state.peek(), &input.read());
        if let Some(synced) = synced {
            state.set(synced);
        }
    });
    let ctx = Ctx { state, input, on_change };

    let s = state.read().clone();
    let entries = h::menu_entries(&s);
    let prompt = s.prompt.clone();
    let editing = prompt.as_ref().is_some_and(|p| !matches!(p.kind, PromptKind::Delete(_)));
    let deleting = prompt.as_ref().is_some_and(|p| matches!(p.kind, PromptKind::Delete(_)));
    let title = prompt.as_ref().map(h::prompt_title).unwrap_or_default();
    let description = prompt.as_ref().map(|p| h::prompt_description(p, &s)).unwrap_or_default();
    let submit = prompt.as_ref().map(h::prompt_submit).unwrap_or_default();
    let name = prompt.as_ref().map(|p| p.name.clone());
    let error = prompt.as_ref().and_then(|p| p.error).map(|e| e.message().to_string());

    rsx! {
        div { "data-scope": h::SCOPE, "data-part": "root",
            Menu {
                items: entries,
                on_select: move |value: String| {
                    let event = h::menu_event(&value, &ctx.state.peek());
                    if let Some(event) = event {
                        ctx.send(event);
                    }
                },
                MenuTrigger {
                    span { "data-scope": h::SCOPE, "data-part": "current", {h::trigger_text(&s)} }
                    Icon { name: IconName::ChevronDown, size: 14 }
                }
                MenuContent {
                    for view in s.views.iter() {
                        MenuItem { key: "{view.id}", value: h::view_value(&view.id),
                            span { "data-scope": h::SCOPE, "data-part": "check", "aria-hidden": "true",
                                if s.active.as_deref() == Some(view.id.as_str()) {
                                    Icon { name: IconName::Check, size: 14 }
                                }
                            }
                            span { "data-scope": h::SCOPE, "data-part": "name", "{view.name}" }
                            if s.default_view.as_deref() == Some(view.id.as_str()) {
                                span { "data-scope": h::SCOPE, "data-part": "default-mark", "Default" }
                            }
                        }
                    }
                    if s.views.is_empty() {
                        p { "data-scope": h::SCOPE, "data-part": "empty", "No saved views yet" }
                    }
                    div { role: "separator", "data-scope": h::SCOPE, "data-part": "separator" }
                    MenuItem { value: h::SAVE_AS, "Save current view as…" }
                    MenuItem { value: h::RENAME, "Rename…" }
                    MenuItem { value: h::TOGGLE_DEFAULT, {h::toggle_default_text(&s)} }
                    MenuItem { value: h::DELETE, "Delete…" }
                }
            }
            Dialog {
                open: Some(prompt.is_some()),
                on_open_change: move |open: bool| {
                    if !open {
                        ctx.send(SavedViewsEvent::ClosePrompt);
                    }
                },
                DialogContent {
                    form {
                        "data-scope": h::SCOPE,
                        "data-part": "form",
                        onsubmit: move |e| {
                            e.prevent_default();
                            ctx.send(SavedViewsEvent::Submit);
                        },
                        DialogTitle { "{title}" }
                        DialogDescription { "{description}" }
                        if editing {
                            Input {
                                label: "Name",
                                value: name,
                                error,
                                on_input: move |text: String| ctx.send(SavedViewsEvent::EditName(text)),
                            }
                        }
                        DialogActions {
                            DialogClose { variant: ButtonVariant::Ghost, "Cancel" }
                            if deleting {
                                Button { button_type: ButtonType::Submit, variant: ButtonVariant::Danger, "{submit}" }
                            } else {
                                Button { button_type: ButtonType::Submit, "{submit}" }
                            }
                        }
                    }
                }
            }
        }
    }
}
