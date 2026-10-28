//! `<Callout>`: a tinted box. Each kind has a fixed jewel and icon.

use leptos::prelude::*;

use crate::icon::Icon;

pub use hydroxy_headless::callout::CalloutKind;

/// Tinted callout. `role="note"`; a `live` danger callout is `role="alert"` and is announced.
#[component]
pub fn Callout(
    /// See [`CalloutKind`].
    #[prop(optional)]
    kind: CalloutKind,
    /// Optional bold first line.
    #[prop(optional, into)]
    title: Option<String>,
    /// Announce a danger callout when it appears. Ignored for other kinds.
    #[prop(optional)]
    live: bool,
    /// The body.
    children: Children,
) -> impl IntoView {
    let role = kind.role(live);
    view! {
        <div data-scope="callout" data-part="root" data-kind=kind.as_str() role=role>
            <span data-scope="callout" data-part="icon">
                <Icon name=kind.icon() size=18 />
            </span>
            <div data-scope="callout" data-part="content">
                {title.map(|t| view! { <p data-scope="callout" data-part="title">{t}</p> })}
                <div data-scope="callout" data-part="body">{children()}</div>
            </div>
        </div>
    }
}
