//! Server-render every docs demo once, so a panic or empty view in any page fails CI.
#![allow(clippy::expect_used, reason = "test code: a poisoned lock should fail the test")]

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

/// Each demo renders under its own owner, which is then dropped on another thread: a
/// multi-threaded server can dispose a request's owner on any worker, and thread-bound
/// values (Leptos `new_local` storage) panic there.
#[test]
fn every_demo_renders_to_non_empty_html() {
    let root = server_owner();
    let failed: Vec<String> = DOCS
        .iter()
        .filter_map(|doc| {
            let owner = root.child();
            let html = catch_unwind(AssertUnwindSafe(|| owner.with(|| (doc.demo)().to_html())));
            let empty = html.map_or(true, |h| h.trim().is_empty());
            let dropped = std::thread::spawn(move || drop(owner)).join().is_ok();
            match (empty, dropped) {
                (false, true) => None,
                (true, _) => Some(format!("{} (panicked or rendered nothing)", doc.slug)),
                (false, false) => Some(format!("{} (panicked when dropped on another thread)", doc.slug)),
            }
        })
        .collect();
    assert!(failed.is_empty(), "failing demos: {failed:#?}");
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
