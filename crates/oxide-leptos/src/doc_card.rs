//! `<DocCard>`: a document in a grid: type tile, title, snippet, optional pin and footer.

use leptos::prelude::*;

use crate::icon::{Icon, IconName};

pub use oxide_headless::jewel::Jewel;

/// A document card. With `on_click` the whole card opens the document (Enter or Space).
#[component]
pub fn DocCard(
    /// The document's title.
    #[prop(into)]
    title: String,
    /// What kind of document, such as "Summary" or "Interview".
    #[prop(optional, into)]
    type_label: Option<String>,
    /// Type tile icon. Default: a document.
    #[prop(default = IconName::FileText)]
    icon: IconName,
    /// Type tile colour. Default amethyst.
    #[prop(default = Jewel::Amethyst)]
    jewel: Jewel,
    /// A few lines of the document, clamped to three.
    #[prop(optional, into)]
    snippet: Option<String>,
    /// Pinned state. The pin button shows only with `on_toggle_pin`.
    #[prop(optional, into)]
    pinned: MaybeProp<bool>,
    /// Shows a pin button and runs when it's pressed.
    #[prop(optional, into)]
    on_toggle_pin: Option<Callback<()>>,
    /// Opens the document.
    #[prop(optional, into)]
    on_click: Option<Callback<()>>,
    /// Footer content, such as an avatar and a date.
    #[prop(optional)]
    children: Option<Children>,
) -> impl IntoView {
    let clickable = on_click.is_some();
    let open = move || {
        if let Some(cb) = on_click {
            cb.run(());
        }
    };
    let is_pinned = move || pinned.get().unwrap_or(false);
    view! {
        <article
            data-scope="doc-card"
            data-clickable=clickable.then_some("")
            role=clickable.then_some("button")
            tabindex=clickable.then_some(0)
            on:click=move |_| open()
            on:keydown=move |ev| {
                if clickable && (ev.key() == "Enter" || ev.key() == " ") {
                    ev.prevent_default();
                    open();
                }
            }
        >
            <div data-part="top">
                <span data-part="icon" data-jewel=jewel.as_str()>
                    <Icon name=icon size=16 />
                </span>
                {type_label.map(|t| view! { <span data-part="type">{t}</span> })}
                {on_toggle_pin
                    .map(|cb| {
                        view! {
                            <button
                                type="button"
                                data-part="pin"
                                aria-pressed=move || is_pinned().to_string()
                                aria-label=move || if is_pinned() { "Unpin" } else { "Pin" }
                                title=move || if is_pinned() { "Unpin" } else { "Pin" }
                                on:click=move |ev| {
                                    ev.stop_propagation();
                                    cb.run(());
                                }
                                on:keydown=|ev| ev.stop_propagation()
                            >
                                <Icon name=IconName::Star size=15 />
                            </button>
                        }
                    })}
            </div>
            <h3 data-part="title">{title}</h3>
            {snippet.map(|s| view! { <p data-part="snippet">{s}</p> })}
            {children.map(|footer| view! { <div data-part="footer">{footer()}</div> })}
        </article>
    }
}
