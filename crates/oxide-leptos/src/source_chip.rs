//! `<SourceChip>` and `<SourceRow>`: mono chips naming the sources behind an answer.

use leptos::prelude::*;

use crate::icon::{Icon, IconName};

/// One source, such as a file name. A button when `on_click` is set.
#[component]
pub fn SourceChip(
    /// Leading icon. Default: a document.
    #[prop(default = IconName::FileText)]
    icon: IconName,
    /// Opens the source.
    #[prop(optional, into)]
    on_click: Option<Callback<()>>,
    /// The source's name.
    children: Children,
) -> impl IntoView {
    match on_click {
        Some(cb) => view! {
            <button type="button" data-scope="source-chip" on:click=move |_| cb.run(())>
                <Icon name=icon size=12 />
                {children()}
            </button>
        }
        .into_any(),
        None => view! {
            <span data-scope="source-chip">
                <Icon name=icon size=12 />
                {children()}
            </span>
        }
        .into_any(),
    }
}

/// A wrapping row of source chips.
#[component]
pub fn SourceRow(
    /// Accessible name for the group. Default: "Sources".
    #[prop(default = "Sources".into(), into)]
    label: String,
    /// The chips.
    children: Children,
) -> impl IntoView {
    view! { <div data-scope="source-row" role="group" aria-label=label>{children()}</div> }
}
