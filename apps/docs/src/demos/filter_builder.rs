use leptos::prelude::*;
use hydroxy_leptos::filter_builder::*;

fn fields() -> Vec<FilterField> {
    vec![
        FilterField::new("title", "Title", FilterFieldKind::Text),
        FilterField::new("score", "Score", FilterFieldKind::Number),
        FilterField::new(
            "status",
            "Status",
            FilterFieldKind::Choice(vec![Choice::new("open", "Open"), Choice::new("review", "In review"), Choice::new("done", "Done")]),
        ),
    ]
}

const RECORDS: [(&str, &str, &str); 5] = [
    ("Weekly digest", "12", "open"),
    ("Board update", "30", "review"),
    ("Tag interviews", "7", "done"),
    ("Import tickets", "18", "open"),
    ("Source check", "25", "review"),
];

pub fn demo() -> AnyView {
    // Uncontrolled, and tell the page about every edit.
    let (filter, set_filter) = signal(FilterValue::default());
    let shown = move || {
        let f = filter.get();
        RECORDS
            .iter()
            .filter(|(title, score, status)| {
                f.matches(|field| match field {
                    "title" => Some((*title).to_string()),
                    "score" => Some((*score).to_string()),
                    "status" => Some((*status).to_string()),
                    _ => None,
                })
            })
            .map(|(title, score, status)| view! { <li>{format!("{title} · score {score} · {status}")}</li> })
            .collect_view()
    };

    view! {
        <div style="display: grid; gap: var(--space-4); max-width: 760px;">
            <FilterBuilder
                fields=fields()
                default_filter=FilterValue {
                    join: FilterJoin::And,
                    rules: vec![FilterRule { id: 0, field: "score".into(), op: FilterOp::Gte, value: "10".into() }],
                }
                on_change=move |f| set_filter.set(f)
            />
            <ul style="margin: 0; padding-left: var(--space-5); font: var(--text-sm); color: var(--text-2);">{shown}</ul>
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "filter-builder",
    name: "Filter builder",
    group: "Data",
    summary: "Build a filter from rows of field, operator and value. The operators depend on the field's type, and the result is a plain value you can store or test records against.",
    demo,
    leptos: include_str!("filter_builder.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/filter_builder.rs"),
    parts: &[
        &["FilterBuilder", "root", "role=group wrapper named Filter"],
        &["FilterBuilder", "join", "Match all / Match any. Shown with two or more conditions"],
        &["FilterBuilder", "rules, rule", "<ul> of <li role=group> rows, each named Condition N"],
        &["FilterBuilder", "field, op, value", "Select, Select and Input (or Select for a choice field). Labels are visually hidden and name the control per condition"],
        &["FilterBuilder", "remove", "IconButton that removes the condition"],
        &["FilterBuilder", "footer", "Holds the Add condition button"],
        &["FilterBuilder", "empty", "Shown when there are no conditions"],
    ],
    keys: &[
        &["Tab", "Moves between the fields, values and buttons of each row"],
        &["Arrow keys / typing", "Choose from a menu, as in a native select"],
    ],
    props: &[
        &["fields", "Vec<FilterField>", "Required. id, label and kind: Text, Number or Choice(options)"],
        &["filter", "Option<FilterValue>", "Controlled filter. Pair with on_change"],
        &["default_filter", "FilterValue", "Initial filter when uncontrolled"],
        &["on_change", "Callback<FilterValue>", "The whole filter after every edit: join plus rules (id, field, op, value)"],
    ],
};
