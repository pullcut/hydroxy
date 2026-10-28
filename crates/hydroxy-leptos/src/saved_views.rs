//! `<SavedViews>`: a menu of saved filter presets with save-as, rename,
//! delete and set-as-default. Built from [`Menu`] and [`Dialog`]. It keeps
//! names and ids only; the owner stores the filters themselves.

use hydroxy_headless::saved_views::{
    self as h, PromptKind, SavedViewsEvent, SavedViewsInput, SavedViewsMachine, SavedViewsState,
};
use hydroxy_statechart::Machine;
use leptos::prelude::*;

use crate::button::{Button, ButtonType, ButtonVariant};
use crate::dialog::{Dialog, DialogActions, DialogClose, DialogContent, DialogDescription, DialogTitle};
use crate::icon::{Icon, IconName};
use crate::input::Input;
use crate::menu::{Menu, MenuContent, MenuItem, MenuTrigger};

pub use hydroxy_headless::saved_views::{SavedView, SavedViewsCommand, apply};

/// The text for a toggle between "Set as default" and "Remove as default".
fn toggle_text(state: RwSignal<SavedViewsState>) -> impl Fn() -> &'static str {
    move || state.with(h::toggle_default_text)
}

/// A dropdown of saved views. Every change is reported through `on_change`.
#[allow(clippy::too_many_lines, reason = "one view! tree; splitting it only adds prop plumbing")]
#[component]
pub fn SavedViews(
    /// Controlled views. Pair with `on_change` and [`apply`].
    #[prop(optional, into)]
    views: MaybeProp<Vec<SavedView>>,
    /// Initial views when uncontrolled.
    #[prop(optional, into)]
    default_views: Vec<SavedView>,
    /// Controlled active view id. Pair with `on_change`.
    #[prop(optional, into)]
    active: MaybeProp<String>,
    /// Initially active view id when uncontrolled.
    #[prop(optional, into)]
    default_active: Option<String>,
    /// The view to open with, such as one the owner loaded from storage.
    #[prop(optional, into)]
    default_view: Option<String>,
    /// Called with every change: a view created, renamed or deleted, a new active view, a new default.
    #[prop(optional, into)]
    on_change: Option<Callback<SavedViewsCommand>>,
) -> impl IntoView {
    let input = Memo::new(move |_| SavedViewsInput {
        views: views.get(),
        default_views: default_views.clone(),
        active: active.get(),
        default_active: default_active.clone(),
        default_view: default_view.clone(),
    });
    let state = RwSignal::new(input.with_untracked(SavedViewsMachine::initial));
    Effect::new(move |_| {
        let synced = input.with(|i| state.with_untracked(|s| SavedViewsMachine::sync(s, i)));
        if let Some(synced) = synced {
            state.set(synced);
        }
    });
    let send = move |event: SavedViewsEvent| {
        let Some(step) = input
            .try_with_untracked(|i| state.try_with_untracked(|s| SavedViewsMachine::transition(s, event, i)))
            .flatten()
        else {
            return;
        };
        if let Some(next) = step.state {
            state.set(next);
        }
        if let Some(cb) = on_change {
            for command in step.commands {
                cb.run(command);
            }
        }
    };

    let entries = Signal::derive(move || state.with(h::menu_entries));
    let ids = Memo::new(move |_| state.with(|s| s.views.iter().map(|v| v.id.clone()).collect::<Vec<_>>()));
    let prompt = move || state.with(|s| s.prompt.clone());
    let editing = move || prompt().is_some_and(|p| !matches!(p.kind, PromptKind::Delete(_)));
    let toggle = toggle_text(state);
    let deleting = move || prompt().is_some_and(|p| matches!(p.kind, PromptKind::Delete(_)));
    let submit_label = move || prompt().map(|p| h::prompt_submit(&p));

    let row = move |id: String| {
        let (for_name, for_active, for_default) = (id.clone(), id.clone(), id.clone());
        let name = move || {
            state.with(|s| s.views.iter().find(|v| v.id == for_name).map(|v| v.name.clone()).unwrap_or_default())
        };
        let is_active = move || state.with(|s| s.active.as_deref() == Some(for_active.as_str()));
        let is_default = move || state.with(|s| s.default_view.as_deref() == Some(for_default.as_str()));
        view! {
            <MenuItem value=h::view_value(&id)>
                <span data-scope=h::SCOPE data-part="check" aria-hidden="true">
                    <Show when=is_active>
                        <Icon name=IconName::Check size=14 />
                    </Show>
                </span>
                <span data-scope=h::SCOPE data-part="name">{name}</span>
                <Show when=is_default>
                    <span data-scope=h::SCOPE data-part="default-mark">"Default"</span>
                </Show>
            </MenuItem>
        }
    };

    view! {
        <div data-scope=h::SCOPE data-part="root">
            <Menu
                items=entries
                on_select=move |value: String| {
                    if let Some(event) = state.with_untracked(|s| h::menu_event(&value, s)) {
                        send(event);
                    }
                }
            >
                <MenuTrigger>
                    <span data-scope=h::SCOPE data-part="current">{move || state.with(h::trigger_text)}</span>
                    <Icon name=IconName::ChevronDown size=14 />
                </MenuTrigger>
                <MenuContent>
                    <For each=move || ids.get() key=String::clone children=row />
                    <Show when=move || state.with(|s| s.views.is_empty())>
                        <p data-scope=h::SCOPE data-part="empty">"No saved views yet"</p>
                    </Show>
                    <div role="separator" data-scope=h::SCOPE data-part="separator"></div>
                    <MenuItem value=h::SAVE_AS>"Save current view as…"</MenuItem>
                    <MenuItem value=h::RENAME>"Rename…"</MenuItem>
                    <MenuItem value=h::TOGGLE_DEFAULT>{toggle}</MenuItem>
                    <MenuItem value=h::DELETE>"Delete…"</MenuItem>
                </MenuContent>
            </Menu>
            <Dialog
                open=Signal::derive(move || prompt().is_some())
                on_open_change=move |open: bool| {
                    if !open {
                        send(SavedViewsEvent::ClosePrompt);
                    }
                }
            >
                <DialogContent>
                    <form
                        data-scope=h::SCOPE
                        data-part="form"
                        on:submit=move |ev| {
                            ev.prevent_default();
                            send(SavedViewsEvent::Submit);
                        }
                    >
                        <DialogTitle>{move || prompt().map(|p| h::prompt_title(&p))}</DialogTitle>
                        <DialogDescription>
                            {move || prompt().map(|p| state.with(|s| h::prompt_description(&p, s)))}
                        </DialogDescription>
                        <Show when=editing>
                            <Input
                                label="Name"
                                value=Signal::derive(move || prompt().map(|p| p.name))
                                error=Signal::derive(move || prompt().and_then(|p| p.error).map(|e| e.message().to_string()))
                                on_input=move |text: String| send(SavedViewsEvent::EditName(text))
                            />
                        </Show>
                        <DialogActions>
                            <DialogClose variant=ButtonVariant::Ghost>"Cancel"</DialogClose>
                            <Show
                                when=deleting
                                fallback=move || view! {
                                    <Button button_type=ButtonType::Submit>{submit_label}</Button>
                                }
                            >
                                <Button button_type=ButtonType::Submit variant=ButtonVariant::Danger>
                                    {submit_label}
                                </Button>
                            </Show>
                        </DialogActions>
                    </form>
                </DialogContent>
            </Dialog>
        </div>
    }
}
