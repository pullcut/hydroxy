//! hydroxy docs site. Rendered on the server by axum + Leptos, then hydrated in the
//! browser, so pages arrive as full HTML and every demo is interactive once loaded.
//!
//! Run from this folder: `cargo leptos watch` (port 8090).
//! Add a component page: a demo module exporting `DOC`, listed in `app::DOCS`.

#![recursion_limit = "256"] // Leptos view types nest deeply; release builds need the headroom.
#![allow(clippy::wildcard_imports, reason = "demos are shown as source; prelude globs are what users write")]
#![allow(clippy::must_use_candidate, reason = "components are called by `view!`, not by hand")]

pub mod app;
mod demos;
mod doc;
mod guide;
mod home;
// Server builds only: with `hydrate` too (as `--all-features` gives), the executor is the browser one.
#[cfg(all(test, feature = "ssr", not(feature = "hydrate")))]
mod render_tests;

/// Browser entry point: hydrate the server-rendered page.
#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(app::App);
}
