//! `AccessEditor`: a sharing panel with a role per person and add-by-email.

use dioxus::prelude::*;
use oxide_headless::access_editor::{
    self as h, AccessEditorCommand, AccessEditorEvent, AccessEditorInput, AccessEditorMachine, Person,
};
use oxide_statechart::Machine;

use crate::avatar::Avatar;
use crate::button::Button;
use crate::icon::{Icon, IconName};
use crate::select::{Select, SelectOption};

pub use h::Person as AccessPerson;

/// A list of people with one role each, and a field to add someone by email.
/// The owner's role is fixed and the owner has no remove button.
#[component]
#[allow(clippy::too_many_lines, reason = "one rsx! tree; splitting it only adds prop plumbing")]
pub fn AccessEditor(
    /// Accessible name for the whole panel, such as "Who has access".
    label: String,
    /// Roles someone can be given, in display order. Leave the owner's role out.
    roles: Vec<SelectOption>,
    /// Role for people added by email. Defaults to the last role.
    new_role: Option<String>,
    /// Controlled people. Pair with `on_people_change`.
    #[props(default)]
    people: ReadSignal<Option<Vec<Person>>>,
    /// Initial people when uncontrolled.
    #[props(default)]
    default_people: Vec<Person>,
    /// Block edits.
    #[props(default)]
    disabled: ReadSignal<bool>,
    /// Called with the new list whenever it changes or a change is requested.
    on_people_change: Option<EventHandler<Vec<Person>>>,
) -> Element {
    let ids = use_hook(|| h::ids(&oxide_headless::next_id(h::SCOPE)));
    let values: Vec<String> = roles.iter().map(|r| r.value.clone()).collect();
    let new_role = new_role.or_else(|| values.last().cloned()).unwrap_or_default();
    let input = use_memo(move || AccessEditorInput {
        people: people(),
        default_people: default_people.clone(),
        roles: values.clone(),
        new_role: new_role.clone(),
        disabled: disabled(),
    });
    let mut state = use_signal(|| AccessEditorMachine::initial(&input.peek()));
    use_effect(move || {
        let synced = AccessEditorMachine::sync(&state.peek(), &input.read());
        if let Some(synced) = synced {
            state.set(synced);
        }
    });
    let mut live = use_signal(String::new);

    let mut send = move |event: AccessEditorEvent| {
        let step = AccessEditorMachine::transition(&state.peek(), event, &input.peek());
        if let Some(next) = step.state {
            state.set(next);
        }
        if let Some(text) = h::last_announcement(&step.commands) {
            live.set(text);
        }
        for command in step.commands {
            if let (AccessEditorCommand::PeopleChange(list), Some(handler)) = (command, on_people_change) {
                handler.call(list);
            }
        }
    };

    let current = state.read().clone();
    let is_disabled = disabled();
    let described_by = ids.error.clone();

    rsx! {
        section {
            "data-scope": h::SCOPE,
            "data-part": "root",
            "aria-label": "{label}",
            "data-disabled": is_disabled.then_some(""),
            div { "data-scope": h::SCOPE, "data-part": "form",
                label { "data-scope": h::SCOPE, "data-part": "label", r#for: "{ids.control}", "Add people by email" }
                div {
                    "data-scope": h::SCOPE,
                    "data-part": "field",
                    "data-invalid": current.error.is_some().then_some(""),
                    input {
                        r#type: "text",
                        inputmode: "email",
                        autocomplete: "off",
                        id: "{ids.control}",
                        "data-scope": h::SCOPE,
                        "data-part": "control",
                        placeholder: "name@company.com",
                        disabled: is_disabled,
                        "aria-invalid": current.error.is_some().then_some("true"),
                        "aria-describedby": described_by,
                        value: "{current.draft}",
                        oninput: move |e| send(AccessEditorEvent::SetDraft(e.value())),
                        onkeydown: move |e| {
                            if let Some(event) = h::keydown(&e.key().to_string()) {
                                e.prevent_default();
                                send(event);
                            }
                        },
                    }
                    Button { disabled: is_disabled, on_click: move |_| send(AccessEditorEvent::Add), "Add" }
                }
                p { "data-scope": h::SCOPE, "data-part": "error", id: "{ids.error}",
                    if let Some(error) = current.error {
                        {h::error_text(error)}
                    }
                }
            }
            ul { "data-scope": h::SCOPE, "data-part": "list",
                for person in current.people.iter().cloned() {
                    {
                        let title = person.display_name().to_string();
                        let owner = person.owner;
                        let role_text = roles
                            .iter()
                            .find(|o| o.value == person.role)
                            .map_or_else(|| "Owner".to_string(), |o| o.label.clone());
                        let email = person.email.clone();
                        let remove_email = person.email.clone();
                        rsx! {
                            li {
                                key: "{person.email}",
                                "data-scope": h::SCOPE,
                                "data-part": "person",
                                "data-owner": owner.then_some(""),
                                Avatar { initials: h::initials(&person), jewel: h::jewel_for(&person.email), size: 32, title: title.clone() }
                                span { "data-scope": h::SCOPE, "data-part": "who",
                                    span { "data-scope": h::SCOPE, "data-part": "name", "{title}" }
                                    if person.name.is_some() {
                                        span { "data-scope": h::SCOPE, "data-part": "email", "{person.email}" }
                                    }
                                }
                                span { "data-scope": h::SCOPE, "data-part": "role",
                                    if owner {
                                        span { "data-scope": h::SCOPE, "data-part": "fixed-role", "{role_text}" }
                                    } else {
                                        Select {
                                            label: h::role_label(&person),
                                            options: roles.clone(),
                                            value: person.role.clone(),
                                            disabled: is_disabled,
                                            on_change: move |role| send(AccessEditorEvent::SetRole { email: email.clone(), role }),
                                        }
                                    }
                                }
                                if !owner {
                                    button {
                                        r#type: "button",
                                        "data-scope": h::SCOPE,
                                        "data-part": "remove",
                                        "aria-label": h::remove_label(&person),
                                        title: "Remove",
                                        disabled: is_disabled,
                                        onclick: move |_| send(AccessEditorEvent::Remove(remove_email.clone())),
                                        Icon { name: IconName::X, size: 14 }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            span { "data-scope": h::SCOPE, "data-part": "live", "aria-live": "polite", "{live}" }
        }
    }
}
