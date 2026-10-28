//! One row of the item list.

use super::Ctx;
use super::{FileItem, ROOT};
use crate::icon::{Icon, IconName};
use hydroxy_headless::file_manager::{self as h, FileManagerEvent};
use leptos::either::Either;
use leptos::prelude::*;

#[allow(
    clippy::too_many_lines,
    clippy::needless_pass_by_value,
    reason = "one view! tree, and `impl IntoView` must not borrow"
)]
pub(super) fn item_view(
    ctx: Ctx,
    root: String,
    shown: Memo<Vec<FileItem>>,
    key: String,
    columns: impl Fn() -> usize + Copy + 'static,
) -> impl IntoView {
    let item = {
        let key = key.clone();
        Memo::new(move |_| shown.with(|v| v.iter().find(|i| i.id == key).cloned()))
    };
    let attrs = {
        let root = root.clone();
        Memo::new(move |_| {
            let fallback = FileItem::file(String::new(), String::new(), ROOT, 0);
            ctx.state
                .with(|s| shown.with(|v| item.with(|i| h::item_attrs(&root, s, v, i.as_ref().unwrap_or(&fallback)))))
        })
    };
    let name = move || item.with(|i| i.as_ref().map(|i| i.name.clone()).unwrap_or_default());
    let is_folder = move || item.with(|i| i.as_ref().is_some_and(FileItem::is_folder));
    let rename = move || ctx.state.with(|s| s.renaming.clone().filter(|r| r.id == key_of(item)));
    let (on_click, on_dbl, on_ctx, on_focus, on_key) = (key.clone(), key.clone(), key.clone(), key.clone(), key);
    let error_id = h::rename_error_id(&root);
    let rename_dom_id = h::rename_id(&root);

    let leading = move || {
        if is_folder() {
            Either::Left(view! {
                <span data-scope=h::SCOPE data-part="icon" aria-hidden="true">
                    <Icon name=IconName::Folder size=20 />
                </span>
            })
        } else {
            Either::Right(
                view! { <span data-scope=h::SCOPE data-part="badge" aria-hidden="true">{move || h::badge_text(&name())}</span> },
            )
        }
    };
    let label = {
        let (error_id, rename_dom_id) = (error_id.clone(), rename_dom_id.clone());
        move || match rename() {
            Some(r) => {
                let described = r.error.map(|_| error_id.clone());
                let message = r.error.map(h::NameError::message);
                Either::Left(view! {
                    <span data-scope=h::SCOPE data-part="name" data-editing="">
                        <input
                            id=rename_dom_id.clone()
                            type="text"
                            aria-label="Name"
                            aria-invalid=message.map(|_| "true")
                            aria-describedby=described
                            data-scope=h::SCOPE
                            data-part="rename"
                            prop:value=r.draft
                            on:input=move |ev| ctx.send(FileManagerEvent::RenameInput(event_target_value(&ev)))
                            on:keydown=move |ev| {
                                // Keys here are for the field, not the row around it.
                                ev.stop_propagation();
                                if let Some(event) = h::rename_keydown(&ev.key()) {
                                    ev.prevent_default();
                                    ctx.send(event);
                                }
                            }
                            on:click=|ev| ev.stop_propagation()
                            on:dblclick=|ev| ev.stop_propagation()
                            on:blur=move |_| ctx.send(FileManagerEvent::CommitRename)
                        />
                        {message.map(|m| view! { <span id=error_id.clone() data-scope=h::SCOPE data-part="rename-error" role="alert">{m}</span> })}
                    </span>
                })
            }
            None => Either::Right(view! { <span data-scope=h::SCOPE data-part="name">{name}</span> }),
        }
    };

    view! {
        <li
            role="option"
            id=move || attrs.get().id
            aria-selected=move || attrs.get().selected.to_string()
            tabindex=move || if attrs.get().tab_stop { "0" } else { "-1" }
            data-scope=h::SCOPE
            data-part="item"
            data-kind=move || attrs.get().kind
            data-selected=move || attrs.get().selected.then_some("")
            data-renaming=move || attrs.get().renaming.then_some("")
            on:click=move |ev| {
                if ctx.state.with_untracked(|s| s.renaming.is_none()) {
                    ev.stop_propagation();
                    let extend = h::click_extend(ev.ctrl_key() || ev.meta_key(), ev.shift_key());
                    ctx.send(FileManagerEvent::Click { id: on_click.clone(), extend });
                }
            }
            on:dblclick=move |_| ctx.send(FileManagerEvent::Activate(on_dbl.clone()))
            on:contextmenu=move |_| ctx.send(FileManagerEvent::Context(on_ctx.clone()))
            on:focus=move |_| ctx.send(FileManagerEvent::Focus(on_focus.clone()))
            on:keydown=move |ev| {
                // The menu key opens the menu without a contextmenu event, so pick the row here.
                if ev.key() == "ContextMenu" || (ev.shift_key() && ev.key() == "F10") {
                    ctx.send(FileManagerEvent::Context(on_key.clone()));
                }
                let view = ctx.state.with_untracked(|s| s.view);
                let event = h::item_keydown(
                    &ev.key(),
                    ev.ctrl_key() || ev.meta_key(),
                    ev.shift_key(),
                    ev.alt_key(),
                    columns(),
                    &on_key,
                    view,
                );
                if let Some(event) = event {
                    ev.prevent_default();
                    ctx.send(event);
                }
            }
        >
            {leading}
            {label}
            <span data-scope=h::SCOPE data-part="modified">{move || item.with(|i| i.as_ref().map(|i| i.modified.clone()))}</span>
            <span data-scope=h::SCOPE data-part="size">{move || item.with(|i| i.as_ref().map(h::size_text))}</span>
        </li>
    }
}

/// The id of the item a memo holds, or an empty string.
fn key_of(item: Memo<Option<FileItem>>) -> String {
    item.with(|i| i.as_ref().map(|i| i.id.clone()).unwrap_or_default())
}
