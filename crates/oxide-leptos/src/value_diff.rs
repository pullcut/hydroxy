//! `<ValueDiff>`: before → after. The old value is struck through.

use leptos::prelude::*;

/// A change from one value to another, such as a renamed status or a moved date.
#[component]
pub fn ValueDiff(
    /// The old value.
    #[prop(into)]
    from: String,
    /// The new value.
    #[prop(into)]
    to: String,
) -> impl IntoView {
    view! {
        <span data-scope="value-diff">
            <span data-part="sr">"Changed from "</span>
            <del data-part="from">{from}</del>
            <span data-part="arrow" aria-hidden="true">"→"</span>
            <span data-part="sr">" to "</span>
            <ins data-part="to" style="text-decoration: none;">{to}</ins>
        </span>
    }
}
