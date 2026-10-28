//! Dioxus hooks. One page-wide [`Registry`] fed by a `document` keydown
//! listener in the webview (through `document::eval`, so it works on web,
//! desktop and mobile). Components add shortcuts with [`use_hotkey`].
//!
//! ```no_run
//! use dioxus::prelude::*;
//! use oxide_hotkeys::dioxus::use_hotkey;
//!
//! #[component]
//! fn Editor() -> Element {
//!     let mut saved = use_signal(|| 0);
//!     use_hotkey("Mod+S", move || saved += 1);
//!     rsx! { p { "Saved {saved} times" } }
//! }
//! ```
//!
//! `preventDefault` has to be decided in the webview before Rust sees the key,
//! so the script keeps a copy of the chords that could match next
//! ([`Registry::armed`]) and checks those. `stop_propagation` is not supported here.

use std::cell::RefCell;

use dioxus::prelude::*;

use crate::{Hotkey, HotkeyId, HotkeyInfo, HotkeyOptions, KeyEvent, KeyMatch, KeyPress, Modifier, Platform, Registry};

struct Global {
    registry: Registry<Callback<()>>,
    link: Option<document::Eval>,
}

thread_local! {
    static GLOBAL: RefCell<Global> = RefCell::new(Global { registry: Registry::default(), link: None });
}

static VERSION: GlobalSignal<u64> = Signal::global(|| 0);

/// Change the registry, then tell the webview and any cheat sheet.
fn update<T>(f: impl FnOnce(&mut Registry<Callback<()>>) -> T) -> T {
    let out = GLOBAL.with_borrow_mut(|g| f(&mut g.registry));
    push();
    *VERSION.write() += 1;
    out
}

type ArmedWire = (String, bool, [u8; 4], bool, Option<String>);

/// Send the chords that could match next, and the selectors to measure, to the webview.
fn push() {
    GLOBAL.with_borrow(|g| {
        let Some(link) = &g.link else { return };
        let armed: Vec<ArmedWire> = g
            .registry
            .armed()
            .into_iter()
            .map(|a| {
                let c = &a.chord;
                let (key, is_code) = match &c.key {
                    KeyMatch::Key(k) => (k.clone(), false),
                    KeyMatch::Code(k) => (k.clone(), true),
                };
                let shift = if c.has(Modifier::Shift) {
                    1
                } else {
                    u8::from(!is_code && crate::hotkey::shift_implied(&key)) * 2
                };
                let mods = [u8::from(c.has(Modifier::Ctrl)), u8::from(c.has(Modifier::Meta)), u8::from(c.has(Modifier::Alt)), shift];
                (key, is_code, mods, a.in_inputs, a.within)
            })
            .collect();
        let _ = link.send((armed, g.registry.selectors()));
    });
}

const SCRIPT: &str = r"
const EDITABLE = __EDITABLE__;
let armed = [], sels = [];
const name = e => {
  const k = e.key;
  if (k === ' ') return 'Space';
  if (k === '+') return 'Plus';
  if ([...k].length !== 1) return k;
  if (/[^\x00-\x7f]/.test(k) && e.altKey) { const m = /^(?:Key|Digit)(.)$/.exec(e.code); if (m) return m[1]; }
  return k.toUpperCase();
};
document.addEventListener('keydown', e => {
  if (e.isComposing) return;
  const t = e.target instanceof Element ? e.target : null;
  const typing = !!(t && t.closest(EDITABLE));
  const depth = s => { const el = t && t.closest(s); if (!el) return -1; let d = 0; for (let n = t; n !== el; n = n.parentElement) d++; return d; };
  const n = name(e);
  if (armed.some(([k, c, m, i, w]) => (c ? e.code === k : n === k) && e.ctrlKey === !!m[0] && e.metaKey === !!m[1]
      && e.altKey === !!m[2] && (m[3] === 2 || e.shiftKey === (m[3] === 1)) && (i || !typing) && (!w || depth(w) >= 0))) {
    e.preventDefault();
  }
  if (['Control', 'Alt', 'Shift', 'Meta'].includes(e.key)) return;
  const flags = [e.ctrlKey, e.altKey, e.shiftKey, e.metaKey, e.repeat, typing].map(b => b ? '1' : '0').join('');
  const msg = [e.key, e.code, flags, String(Math.round(performance.now()))];
  for (const s of sels) msg.push(s, String(depth(s)));
  dioxus.send(msg);
});
dioxus.send(['platform', navigator.platform || navigator.userAgent]);
for (;;) { [armed, sels] = await dioxus.recv(); }
";

/// Start the webview listener once. Client only: called from effects.
fn listen() {
    if GLOBAL.with_borrow(|g| g.link.is_some()) {
        return;
    }
    let script = SCRIPT.replace("__EDITABLE__", &format!("{:?}", crate::EDITABLE_SELECTOR));
    let link = document::eval(&script);
    GLOBAL.with_borrow_mut(|g| g.link = Some(link));
    push();
    dioxus::core::spawn_forever(async move {
        let mut link = link;
        while let Ok(msg) = link.recv::<Vec<String>>().await {
            receive(&msg);
        }
    });
}

fn receive(msg: &[String]) {
    if let [tag, platform] = msg
        && tag == "platform"
    {
        update(|r| r.set_platform(Platform::detect(platform)));
        return;
    }
    let [key, code, flags, ms, rest @ ..] = msg else { return };
    let flag = |i: usize| flags.as_bytes().get(i) == Some(&b'1');
    let event = KeyEvent {
        press: KeyPress { key: key.clone(), code: code.clone(), ctrl: flag(0), alt: flag(1), shift: flag(2), meta: flag(3) },
        repeat: flag(4),
        editable: flag(5),
    };
    let within = |selector: &str| {
        rest.as_chunks::<2>().0.iter().find(|[s, _]| s == selector).and_then(|[_, d]| d.parse::<usize>().ok())
    };
    let now = std::time::Duration::from_millis(ms.parse().unwrap_or(0));
    let hit = GLOBAL.with_borrow_mut(|g| g.registry.dispatch(&event, now, within));
    push();
    if let Some(handler) = hit.and_then(|h| h.handler) {
        handler.call(());
    }
}

/// A registered shortcut. `Copy`, so it can go into closures.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HotkeyHandle(HotkeyId);

impl HotkeyHandle {
    /// Its id.
    #[must_use]
    pub fn id(self) -> HotkeyId {
        self.0
    }

    /// Turn it on or off.
    pub fn set_enabled(self, enabled: bool) {
        update(|r| r.set_enabled(self.0, enabled));
    }
}

/// Run `handler` when `keys` is pressed, with the default [`HotkeyOptions`].
///
/// # Panics
///
/// If `keys` is not a valid shortcut; see [`Hotkey`].
pub fn use_hotkey(keys: &str, handler: impl FnMut() + 'static) -> HotkeyHandle {
    use_hotkey_with(keys, HotkeyOptions::default(), handler)
}

/// Run `handler` when `keys` is pressed, with `options` (read on first render only).
///
/// # Panics
///
/// If `keys` is not a valid shortcut; see [`Hotkey`].
pub fn use_hotkey_with(keys: &str, options: HotkeyOptions, mut handler: impl FnMut() + 'static) -> HotkeyHandle {
    let callback = use_callback(move |()| handler());
    let id = use_hook(|| {
        let hotkey: Hotkey = keys.parse().unwrap_or_else(|e| panic!("invalid hotkey {keys:?}: {e}"));
        GLOBAL.with_borrow_mut(|g| g.registry.register(hotkey, options, callback))
    });
    use_effect(move || {
        listen();
        push();
        *VERSION.write() += 1;
    });
    use_drop(move || {
        update(|r| r.unregister(id));
    });
    HotkeyHandle(id)
}

/// Turn a named scope (see [`HotkeyOptions::scope`]) on or off.
pub fn set_hotkey_scope(scope: &str, active: bool) {
    update(|r| r.set_scope_active(scope, active));
}

/// The platform `Mod` resolves for, once the webview has reported it.
#[must_use]
pub fn hotkey_platform() -> Platform {
    let _ = VERSION.read();
    GLOBAL.with_borrow(|g| g.registry.platform())
}

/// Every registered shortcut, for a cheat sheet. Empty until mounted, so
/// server and client render the same markup first.
#[must_use]
pub fn use_hotkeys() -> Memo<Vec<HotkeyInfo>> {
    let mut mounted = use_signal(|| false);
    use_effect(move || mounted.set(true));
    use_memo(move || {
        let _ = VERSION.read();
        if mounted() { GLOBAL.with_borrow(|g| g.registry.list()) } else { Vec::new() }
    })
}
