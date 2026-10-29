//! Server-render every docs demo once, so a panic or empty view in any page fails CI.

use std::collections::HashSet;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::{Arc, Mutex};

use leptos::prelude::*;

use crate::demos::DOCS;

/// An owner that renders as the server does, so `is_server()` holds and browser-only
/// code (window listeners, observers) is skipped. `leptos_axum` sets up the server's
/// shared context while it lists routes; borrow it rather than depend on
/// `hydration_context` directly.
fn server_owner() -> Owner {
    let slot = Arc::new(Mutex::new(None));
    let fill = Arc::clone(&slot);
    let _ = leptos_axum::generate_route_list_with_exclusions_and_ssg_and_context(
        || (),
        None,
        move || {
            *fill.lock().expect("slot lock") = Owner::current_shared_context();
        },
    );
    let context = slot.lock().expect("slot lock").take();
    assert!(context.is_some(), "leptos_axum provided no shared context");
    Owner::new_root(context)
}

#[test]
fn every_demo_renders_to_non_empty_html() {
    let owner = server_owner();
    let failed: Vec<&str> = DOCS
        .iter()
        .filter(|doc| {
            let html = catch_unwind(AssertUnwindSafe(|| owner.with(|| (doc.demo)().to_html())));
            html.map_or(true, |h| h.trim().is_empty())
        })
        .map(|doc| doc.slug)
        .collect();
    assert!(failed.is_empty(), "demos that panicked or rendered nothing: {failed:?}");
}

#[test]
fn slugs_are_unique() {
    let mut seen = HashSet::new();
    for doc in DOCS {
        assert!(seen.insert(doc.slug), "duplicate slug `{}`", doc.slug);
    }
}

#[test]
fn table_rows_are_non_empty() {
    for doc in DOCS {
        for (table, rows) in [("parts", doc.parts), ("keys", doc.keys), ("props", doc.props)] {
            for row in rows {
                assert!(
                    !row.is_empty() && row.iter().any(|cell| !cell.trim().is_empty()),
                    "`{}` has an empty {table} row",
                    doc.slug
                );
            }
        }
    }
}
