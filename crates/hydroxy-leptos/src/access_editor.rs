//! `<AccessEditor>`: a sharing panel with a role per person and add-by-email.

use hydroxy_headless::access_editor::{
    self as h, AccessEditorCommand, AccessEditorEvent, AccessEditorInput, AccessEditorMachine, AccessEditorState,
    Person,
};
use hydroxy_statechart::Machine;
use leptos::either::Either;
use leptos::prelude::*;

use crate::avatar::Avatar;
use crate::button::Button;
use crate::icon::{Icon, IconName};
use crate::select::{Select, SelectOption};

pub use h::Person as AccessPerson;

/// A list of people with one role each, and a field to add someone by email.
/// The owner's role is fixed and the owner has no remove button.
#[component]
#[allow(
    clippy::too_many_lines,
    clippy::needless_pass_by_value,
    reason = "one view! tree; props are owned by the macro"
)]
pub fn AccessEditor(
    /// Accessible name for the whole panel, such as "Who has access".
    #[prop(into)]
    label: String,
    /// Roles someone can be given, in display order. Leave the owner's role out.
    roles: Vec<SelectOption>,
    /// Role for people added by email. Defaults to the last role.
    #[prop(optional, into)]
    new_role: Option<String>,
    /// Controlled people. Pair with `on_people_change`.
    #[prop(optional, into)]
    people: MaybeProp<Vec<Person>>,
    /// Initial people when uncontrolled.
    #[prop(optional)]
    default_people: Vec<Person>,
    /// Block edits.
    #[prop(optional, into)]
    disabled: Signal<bool>,
    /// Called with the new list whenever it changes or a change is requested.
    #[prop(optional, into)]
    on_people_change: Option<Callback<Vec<Person>>>,
) -> impl IntoView {
    let ids = h::ids(&crate::id::next_id(h::SCOPE));
    let values: Vec<String> = roles.iter().map(|r| r.value.clone()).collect();
    let new_role = new_role.or_else(|| values.last().cloned()).unwrap_or_default();
    let input = Memo::new(move |_| AccessEditorInput {
        people: people.get(),
        default_people: default_people.clone(),
        roles: values.clone(),
        new_role: new_role.clone(),
        disabled: disabled.get(),
    });
    let state = RwSignal::new(input.with_untracked(AccessEditorMachine::initial));
    Effect::new(move |_| {
        let synced = input.with(|i| state.with_untracked(|s| AccessEditorMachine::sync(s, i)));
        if let Some(synced) = synced {
            state.set(synced);
        }
    });
    let live = RwSignal::new(String::new());

    let send = move |event: AccessEditorEvent| {
        let Some(step) = input
            .try_with_untracked(|i| state.try_with_untracked(|s| AccessEditorMachine::transition(s, event, i)))
            .flatten()
        else {
            return;
        };
        if let Some(next) = step.state {
            state.set(next);
        }
        if let Some(text) = h::last_announcement(&step.commands) {
            live.set(text);
        }
        for command in step.commands {
            if let (AccessEditorCommand::PeopleChange(list), Some(cb)) = (command, on_people_change) {
                cb.run(list);
            }
        }
    };

    let role_options = StoredValue::new(roles);
    let control_id = ids.control.clone();
    let row = move |email: String| {
        let control_id = control_id.clone();
        let find = {
            let email = email.clone();
            move || state.with(|s: &AccessEditorState| s.people.iter().find(|p| p.email == email).cloned())
        };
        let person = find()?;
        let (initial, title) = (h::initials(&person), person.display_name().to_string());
        let shown_email = person.name.is_some().then(|| person.email.clone());
        let owner = person.owner;
        let role_text = role_options
            .with_value(|r| r.iter().find(|o| o.value == person.role).map(|o| o.label.clone()))
            .unwrap_or_else(|| "Owner".to_string());
        let control = if owner {
            Either::Left(view! { <span data-scope=h::SCOPE data-part="fixed-role">{role_text}</span> })
        } else {
            let (current_email, label) = (email.clone(), h::role_label(&person));
            Either::Right(view! {
                <Select
                    label=label
                    options=role_options.get_value()
                    value=Signal::derive({
                        let find = find.clone();
                        move || find().map(|p| p.role)
                    })
                    disabled=disabled
                    on_change=move |role| send(AccessEditorEvent::SetRole { email: current_email.clone(), role })
                />
            })
        };
        let remove = (!owner).then(|| {
            let (remove_email, aria) = (email.clone(), h::remove_label(&person));
            view! {
                <button
                    type="button"
                    data-scope=h::SCOPE
                    data-part="remove"
                    aria-label=aria
                    title="Remove"
                    disabled=move || disabled.get()
                    on:click=move |_| {
                        send(AccessEditorEvent::Remove(remove_email.clone()));
                        // The button is gone now; keep focus inside the panel.
                        crate::focus::focus_id(&control_id);
                    }
                >
                    <Icon name=IconName::X size=14 />
                </button>
            }
        });
        Some(view! {
            <li data-scope=h::SCOPE data-part="person" data-owner=owner.then_some("")>
                <Avatar initials=initial jewel=h::jewel_for(&person.email) size=32 title=title.clone() />
                <span data-scope=h::SCOPE data-part="who">
                    <span data-scope=h::SCOPE data-part="name">{title}</span>
                    {shown_email.map(|e| view! { <span data-scope=h::SCOPE data-part="email">{e}</span> })}
                </span>
                <span data-scope=h::SCOPE data-part="role">{control}</span>
                {remove}
            </li>
        })
    };

    view! {
        <section data-scope=h::SCOPE data-part="root" aria-label=label data-disabled=move || disabled.get().then_some("")>
            <div data-scope=h::SCOPE data-part="form">
                <label data-scope=h::SCOPE data-part="label" for=ids.control.clone()>"Add people by email"</label>
                <div data-scope=h::SCOPE data-part="field" data-invalid=move || state.with(|s| s.error.is_some()).then_some("")>
                    <input
                        type="text"
                        inputmode="email"
                        autocomplete="off"
                        id=ids.control.clone()
                        data-scope=h::SCOPE
                        data-part="control"
                        placeholder="name@company.com"
                        disabled=move || disabled.get()
                        aria-invalid=move || state.with(|s| s.error.is_some()).then_some("true")
                        aria-describedby=ids.error.clone()
                        prop:value=move || state.with(|s| s.draft.clone())
                        on:input=move |ev| send(AccessEditorEvent::SetDraft(event_target_value(&ev)))
                        on:keydown=move |ev| {
                            if let Some(event) = h::keydown(&ev.key()) {
                                ev.prevent_default();
                                send(event);
                            }
                        }
                    />
                    <Button disabled=disabled on_click=move |_| send(AccessEditorEvent::Add)>"Add"</Button>
                </div>
                <p data-scope=h::SCOPE data-part="error" id=ids.error.clone()>
                    {move || state.with(|s| s.error).map(h::error_text)}
                </p>
            </div>
            <ul data-scope=h::SCOPE data-part="list">
                <For
                    each=move || state.with(|s| s.people.iter().map(|p| p.email.clone()).collect::<Vec<_>>())
                    key=Clone::clone
                    children=row
                />
            </ul>
            <span data-scope=h::SCOPE data-part="live" aria-live="polite">{move || live.get()}</span>
        </section>
    }
}
