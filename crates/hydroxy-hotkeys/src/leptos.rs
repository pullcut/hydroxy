//! Leptos hooks. One page-wide [`Registry`] listens on `document`; components add
//! shortcuts with [`use_hotkey`] and they go away when the component unmounts.
//! On the server every hook is a no-op.
//!
//! ```no_run
//! use leptos::prelude::*;
//! use hydroxy_hotkeys::HotkeyOptions;
//! use hydroxy_hotkeys::leptos::{use_hotkey, use_hotkey_with};
//!
//! #[component]
//! fn Editor() -> impl IntoView {
//!     let (saved, set_saved) = signal(0);
//!     use_hotkey("Mod+S", move || set_saved.update(|n| *n += 1));
//!     use_hotkey_with("g i", HotkeyOptions::new().description("Go to inbox"), || {});
//!     view! { <p>"Saved " {saved} " times"</p> }
//! }
//! ```

use std::cell::RefCell;
use std::rc::Rc;

use leptos::prelude::*;

use crate::{Hotkey, HotkeyId, HotkeyInfo, HotkeyOptions, Platform, Registry};

type Handler = Rc<dyn Fn()>;

struct Global {
    registry: Registry<Handler>,
    list: ArcRwSignal<Vec<HotkeyInfo>>,
    listening: bool,
}

thread_local! {
    static GLOBAL: RefCell<Global> = RefCell::new(Global {
        registry: Registry::new(detect_platform()),
        list: ArcRwSignal::new(Vec::new()),
        listening: false,
    });
}

fn detect_platform() -> Platform {
    #[cfg(target_arch = "wasm32")]
    if let Some(window) = web_sys::window() {
        let nav = window.navigator();
        return Platform::detect(&nav.platform().or_else(|_| nav.user_agent()).unwrap_or_default());
    }
    Platform::Other
}

/// Change the registry, then publish the new list outside the borrow.
fn update<T>(f: impl FnOnce(&mut Registry<Handler>) -> T) -> T {
    let (out, list, signal) = GLOBAL.with_borrow_mut(|g| {
        let out = f(&mut g.registry);
        (out, g.registry.list(), g.list.clone())
    });
    signal.set(list);
    out
}

/// A registered shortcut. `Copy`, so it can go into closures.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HotkeyHandle(Option<HotkeyId>);

impl HotkeyHandle {
    /// Its id, or `None` on the server.
    #[must_use]
    pub fn id(self) -> Option<HotkeyId> {
        self.0
    }

    /// Turn it on or off, for example from an `Effect` watching a signal.
    pub fn set_enabled(self, enabled: bool) {
        if let Some(id) = self.0 {
            update(|r| r.set_enabled(id, enabled));
        }
    }
}

/// Run `handler` when `keys` is pressed, with the default [`HotkeyOptions`].
///
/// # Panics
///
/// If `keys` is not a valid shortcut; see [`Hotkey`].
pub fn use_hotkey(keys: &str, handler: impl Fn() + 'static) -> HotkeyHandle {
    use_hotkey_with(keys, HotkeyOptions::default(), handler)
}

/// Run `handler` when `keys` is pressed, with `options`.
///
/// # Panics
///
/// If `keys` is not a valid shortcut; see [`Hotkey`].
pub fn use_hotkey_with(keys: &str, options: HotkeyOptions, handler: impl Fn() + 'static) -> HotkeyHandle {
    let hotkey: Hotkey = keys.parse().unwrap_or_else(|e| panic!("invalid hotkey {keys:?}: {e}"));
    if cfg!(not(target_arch = "wasm32")) {
        return HotkeyHandle(None);
    }
    listen();
    let id = update(|r| r.register(hotkey, options, Rc::new(handler)));
    on_cleanup(move || {
        update(|r| r.unregister(id));
    });
    HotkeyHandle(Some(id))
}

/// Turn a named scope (see [`HotkeyOptions::scope`]) on or off.
pub fn set_hotkey_scope(scope: &str, active: bool) {
    if cfg!(target_arch = "wasm32") {
        update(|r| r.set_scope_active(scope, active));
    }
}

/// The platform `Mod` resolves for. [`Platform::Other`] on the server, so read
/// it after mount if the markup depends on it.
#[must_use]
pub fn hotkey_platform() -> Platform {
    GLOBAL.with_borrow(|g| g.registry.platform())
}

/// Every registered shortcut, for a cheat sheet. Empty on the server and during
/// hydration, filled in after mount.
#[must_use]
pub fn use_hotkeys() -> Signal<Vec<HotkeyInfo>> {
    let out = RwSignal::new(Vec::new());
    let list = GLOBAL.with_borrow(|g| g.list.clone());
    Effect::new(move || out.set(list.get()));
    out.into()
}

fn listen() {
    if !GLOBAL.with_borrow_mut(|g| std::mem::replace(&mut g.listening, true)) {
        listen_on_document();
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn listen_on_document() {}

#[cfg(target_arch = "wasm32")]
fn listen_on_document() {
    use std::time::Duration;

    use leptos::wasm_bindgen::JsCast;
    use leptos::wasm_bindgen::closure::Closure;

    use crate::{EDITABLE_SELECTOR, KeyEvent, KeyPress};

    let on_key = Closure::<dyn Fn(web_sys::KeyboardEvent)>::new(|ev: web_sys::KeyboardEvent| {
        if ev.is_composing() {
            return;
        }
        let target = ev.target().and_then(|t| t.dyn_into::<web_sys::Element>().ok());
        let event = KeyEvent {
            press: KeyPress {
                key: ev.key(),
                code: ev.code(),
                ctrl: ev.ctrl_key(),
                alt: ev.alt_key(),
                shift: ev.shift_key(),
                meta: ev.meta_key(),
            },
            repeat: ev.repeat(),
            editable: target.as_ref().is_some_and(|t| t.closest(EDITABLE_SELECTOR).ok().flatten().is_some()),
        };
        let within = |selector: &str| {
            let target = target.as_ref()?;
            let found = target.closest(selector).ok().flatten()?;
            let mut depth = 0;
            let mut node = target.clone();
            while node != found {
                node = node.parent_element()?;
                depth += 1;
            }
            Some(depth)
        };
        let now = web_sys::window().and_then(|w| w.performance()).map_or(0.0, |p| p.now());
        let now = Duration::from_secs_f64(now.max(0.0) / 1000.0);
        let hit = GLOBAL.with_borrow_mut(|g| g.registry.dispatch(&event, now, within));
        if let Some(hit) = hit {
            if hit.prevent_default {
                ev.prevent_default();
            }
            if hit.stop_propagation {
                ev.stop_propagation();
            }
            if let Some(handler) = hit.handler {
                handler();
            }
        }
    });
    if let Some(document) = web_sys::window().and_then(|w| w.document()) {
        let _ = document.add_event_listener_with_callback("keydown", on_key.as_ref().unchecked_ref());
    }
    // Lives as long as the page.
    on_key.forget();
}
