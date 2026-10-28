use hydroxy_leptos::access_editor::{AccessEditor, AccessPerson};
use hydroxy_leptos::select::SelectOption;
use leptos::prelude::*;

pub fn demo() -> AnyView {
    let roles = vec![SelectOption::new("editor", "Can edit"), SelectOption::new("viewer", "Can view")];
    let people = vec![
        AccessPerson::new("ana@hydroxy.dev", "Ana Kowal", "owner").owner(),
        AccessPerson::new("jo@hydroxy.dev", "Jo Marsh", "editor"),
        AccessPerson::new("sam@partner.io", "Sam Lee", "viewer"),
    ];
    let (count, set_count) = signal(people.len());
    let on_change = Callback::new(move |list: Vec<AccessPerson>| set_count.set(list.len()));

    view! {
        <div style="display: grid; gap: var(--space-5); max-width: 520px;">
            <AccessEditor
                label="Who has access"
                roles=roles
                new_role="viewer"
                default_people=people
                on_people_change=on_change
            />
            <p style="margin: 0; font: var(--text-sm); color: var(--text-2);">
                {move || format!("{} people have access", count.get())}
            </p>
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "access-editor",
    name: "Access editor",
    group: "Data",
    summary: "A sharing panel: add people by email, give each a role, and remove them. The owner's role is fixed and the owner cannot be removed.",
    demo,
    leptos: include_str!("access_editor.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/access_editor.rs"),
    parts: &[
        &["AccessEditor", "root", "A labelled section. data-disabled"],
        &[
            "(inside)",
            "form / label / field / control",
            "The add-by-email row. data-invalid on the field while there is an error",
        ],
        &["(inside)", "error", "Why the last add failed. aria-describedby the input"],
        &["(inside)", "list / person", "One li per person. data-owner on the owner"],
        &["(inside)", "who / name / email", "The person's name and, when known, their email"],
        &["(inside)", "role", "A native Select per person, or fixed-role text for the owner"],
        &["(inside)", "remove", "<button aria-label=\"Remove {name}\">. Hidden for the owner"],
        &["(inside)", "live", "Visually hidden aria-live region for adds, removals and errors"],
    ],
    keys: &[
        &["Enter", "In the email field: add the person"],
        &["Tab", "Move through each role select and remove button"],
    ],
    props: &[
        &["label", "String", "Required. Accessible name for the panel"],
        &["roles", "Vec<SelectOption>", "Required. Roles to choose from. Leave the owner's role out"],
        &["new_role", "String", "Role for people added by email. Defaults to the last role"],
        &["people", "Option<Vec<Person>>", "Controlled people. Pair with on_people_change"],
        &["default_people", "Vec<Person>", "Initial people when uncontrolled"],
        &["disabled", "bool", "Block edits"],
        &[
            "on_people_change",
            "Callback<Vec<Person>>",
            "Called with the new list after every add, removal and role change",
        ],
    ],
};
