//! `<InsightNote>`: an amethyst-tinted summary with the mark, a label and source chips.

use leptos::prelude::*;

use crate::icon::{Icon, IconName};
use crate::source_chip::{SourceChip, SourceRow};

/// A short machine-written summary, with the files it came from.
#[component]
pub fn InsightNote(
    /// What the note is, such as "Summary" or "Weekly digest".
    #[prop(default = "Summary".into(), into)]
    label: String,
    /// File names behind the note, shown as source chips.
    #[prop(optional, into)]
    sources: Vec<String>,
    /// Control at the end of the header, such as a "Regenerate" button.
    #[prop(optional)]
    action: Option<Children>,
    /// The note.
    children: Children,
) -> impl IntoView {
    let name = label.clone();
    view! {
        <section data-scope="insight-note" aria-label=name>
            <div data-part="head">
                <Icon name=IconName::Mark size=16 />
                <span data-part="label">{label}</span>
                {action.map(|a| view! { <span data-part="action">{a()}</span> })}
            </div>
            <div data-part="body">{children()}</div>
            {(!sources.is_empty())
                .then(|| {
                    view! {
                        <SourceRow>
                            {sources.into_iter().map(|s| view! { <SourceChip>{s}</SourceChip> }).collect_view()}
                        </SourceRow>
                    }
                })}
        </section>
    }
}
