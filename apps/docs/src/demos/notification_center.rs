use hydroxy_leptos::notification_center::*;
use leptos::prelude::*;

#[component]
pub fn Demo() -> impl IntoView {
    let (opened, set_opened) = signal(String::from("none yet"));

    view! {
        <div style="display: flex; align-items: center; gap: var(--space-5);">
            // Uncontrolled. Opening a notification marks it read; dismissing removes it.
            <NotificationCenter
                placement=Placement::new(Side::Bottom, Align::Start)
                default_items=vec![
                    Notification::new("deploy", "Deploy finished", "2m ago").description("Production is on build 482."),
                    Notification::new("comment", "Maya commented on Q3 summary", "18m ago")
                        .description("\"Can we add the churn numbers?\""),
                    Notification::new("invite", "Lee accepted your invite", "1h ago"),
                    Notification::new("export", "Export is ready", "Yesterday").read(),
                ]
                on_select=move |id| set_opened.set(id)
            />
            <span>"Last opened: " {opened}</span>
        </div>
    }
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "notification-center",
    name: "Notification center",
    group: "Feedback",
    summary: "A bell with an unread count that opens a list of notifications. Mark them read one at a time or all at once, dismiss them, and see an empty state when none are left.",
    demo: || view! { <Demo /> }.into_any(),
    leptos: include_str!("notification_center.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/notification_center.rs"),
    parts: &[
        &["NotificationCenter", "trigger", "Icon button with popovertarget. aria-label includes the unread count"],
        &["", "badge", "The unread count, capped at 99+. Hidden at zero"],
        &["", "content", "popover=auto, role=dialog, labelled by the title"],
        &["", "header / title / mark-all", "Heading row with the Mark all read button"],
        &["", "list / item", "ul of notifications; unread ones have data-unread"],
        &["", "item-button", "Opens the notification and marks it read"],
        &["", "dismiss", "Icon button that removes the notification"],
        &["", "empty", "Shown when the list is empty"],
    ],
    keys: &[
        &["Enter / Space", "On the bell: opens the list. On a row: opens it. On a button: activates it"],
        &["Tab / Shift+Tab", "Moves through rows, dismiss buttons and Mark all read"],
        &["Escape", "Closes the list and returns focus to the bell"],
    ],
    props: &[
        &["items", "Option<Vec<Notification>>", "Controlled notifications. Pair with on_items_change"],
        &["default_items", "Vec<Notification>", "Initial notifications when uncontrolled"],
        &["label", "String", "Accessible name of the bell and heading. Default Notifications"],
        &["placement", "Placement", "Where the list sits. Default below, end-aligned"],
        &["on_items_change", "Callback<Vec<Notification>>", "Called with the new list when it changes or is requested"],
        &["on_select", "Callback<String>", "Called with the id of a notification when it is opened"],
    ],
    events: &[],
};
