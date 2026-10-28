//! The registry: every live shortcut, which one wins a key press, and the list
//! a cheat sheet reads.

use std::collections::HashSet;
use std::time::Duration;

use crate::hotkey::{Chord, Hotkey};
use crate::keys::{KeyPress, Platform, is_modifier_key};

/// How a registered shortcut behaves. Build with the methods; the defaults
/// suit most app shortcuts.
///
/// ```
/// use hydroxy_hotkeys::HotkeyOptions;
/// let options = HotkeyOptions::new().description("Save").group("File").in_inputs(true);
/// assert!(options.prevent_default);
/// ```
#[allow(clippy::struct_excessive_bools, reason = "independent on/off switches")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HotkeyOptions {
    /// What it does, for the cheat sheet.
    pub description: Option<String>,
    /// A heading to file it under in the cheat sheet.
    pub group: Option<String>,
    /// A named scope; it fires only while that scope is active. `None` is global.
    pub scope: Option<String>,
    /// A CSS selector; it fires only while focus is inside a matching element.
    /// The innermost match beats shortcuts further out and global ones.
    pub within: Option<String>,
    /// Off means registered (and listed) but never fires. Default on.
    pub enabled: bool,
    /// Call `preventDefault` when it fires, or when a sequence advances. Default on.
    pub prevent_default: bool,
    /// Call `stopPropagation` when it fires. Default off.
    pub stop_propagation: bool,
    /// Fire while typing in an input, textarea, select or contenteditable. Default off.
    pub in_inputs: bool,
    /// Fire again on auto-repeat while the key is held. Default off.
    pub repeat: bool,
    /// Higher wins when two shortcuts match at the same depth. Default 0.
    pub priority: i32,
    /// Longest gap between the chords of a sequence. Default one second.
    pub timeout: Duration,
}

impl Default for HotkeyOptions {
    fn default() -> Self {
        Self {
            description: None,
            group: None,
            scope: None,
            within: None,
            enabled: true,
            prevent_default: true,
            stop_propagation: false,
            in_inputs: false,
            repeat: false,
            priority: 0,
            timeout: Duration::from_secs(1),
        }
    }
}

impl HotkeyOptions {
    /// The defaults.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
    /// Set [`description`](Self::description).
    #[must_use]
    pub fn description(mut self, text: impl Into<String>) -> Self {
        self.description = Some(text.into());
        self
    }
    /// Set [`group`](Self::group).
    #[must_use]
    pub fn group(mut self, name: impl Into<String>) -> Self {
        self.group = Some(name.into());
        self
    }
    /// Set [`scope`](Self::scope).
    #[must_use]
    pub fn scope(mut self, name: impl Into<String>) -> Self {
        self.scope = Some(name.into());
        self
    }
    /// Set [`within`](Self::within).
    #[must_use]
    pub fn within(mut self, selector: impl Into<String>) -> Self {
        self.within = Some(selector.into());
        self
    }
    /// Set [`enabled`](Self::enabled).
    #[must_use]
    pub fn enabled(mut self, on: bool) -> Self {
        self.enabled = on;
        self
    }
    /// Set [`prevent_default`](Self::prevent_default).
    #[must_use]
    pub fn prevent_default(mut self, on: bool) -> Self {
        self.prevent_default = on;
        self
    }
    /// Set [`stop_propagation`](Self::stop_propagation).
    #[must_use]
    pub fn stop_propagation(mut self, on: bool) -> Self {
        self.stop_propagation = on;
        self
    }
    /// Set [`in_inputs`](Self::in_inputs).
    #[must_use]
    pub fn in_inputs(mut self, on: bool) -> Self {
        self.in_inputs = on;
        self
    }
    /// Set [`repeat`](Self::repeat).
    #[must_use]
    pub fn repeat(mut self, on: bool) -> Self {
        self.repeat = on;
        self
    }
    /// Set [`priority`](Self::priority).
    #[must_use]
    pub fn priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }
    /// Set [`timeout`](Self::timeout).
    #[must_use]
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
}

/// Identifies one registration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct HotkeyId(u64);

/// A key going down, as the adapter saw it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct KeyEvent {
    /// The key and modifiers.
    pub press: KeyPress,
    /// `KeyboardEvent.repeat`.
    pub repeat: bool,
    /// Focus is in something you type into.
    pub editable: bool,
}

/// One registered shortcut, for a cheat sheet or settings page.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HotkeyInfo {
    /// Its id.
    pub id: HotkeyId,
    /// The shortcut.
    pub hotkey: Hotkey,
    /// What it does.
    pub description: Option<String>,
    /// Its cheat-sheet heading.
    pub group: Option<String>,
    /// Its named scope.
    pub scope: Option<String>,
    /// Enabled and in an active scope.
    pub active: bool,
}

/// The winner of a key press. The adapter calls `preventDefault` and friends,
/// then the handler.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dispatch<H> {
    /// Which registration won.
    pub id: HotkeyId,
    /// The handler to run, or `None` when a sequence only advanced.
    pub handler: Option<H>,
    /// Call `preventDefault`.
    pub prevent_default: bool,
    /// Call `stopPropagation`.
    pub stop_propagation: bool,
}

/// A chord that could match the next key press, for adapters that have to
/// decide `preventDefault` before asking the registry (Dioxus' webview).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Armed {
    /// The chord, with `Mod` resolved for the platform.
    pub chord: Chord,
    /// Fires while typing.
    pub in_inputs: bool,
    /// Only inside this selector.
    pub within: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Step {
    Progress,
    Complete,
}

#[derive(Debug)]
struct Entry<H> {
    id: HotkeyId,
    hotkey: Hotkey,
    options: HotkeyOptions,
    handler: H,
    /// Chords of a sequence matched so far.
    progress: usize,
    last: Duration,
}

impl<H> Entry<H> {
    fn advance(&mut self, press: &KeyPress, now: Duration, platform: Platform) -> Option<Step> {
        let chords = self.hotkey.chords();
        if self.progress > 0 && now.saturating_sub(self.last) > self.options.timeout {
            self.progress = 0;
        }
        if chords[self.progress].matches(press, platform) {
            self.progress += 1;
        } else if self.progress > 0 && chords[0].matches(press, platform) {
            self.progress = 1;
        } else {
            self.progress = 0;
            return None;
        }
        self.last = now;
        if self.progress == chords.len() {
            self.progress = 0;
            Some(Step::Complete)
        } else {
            Some(Step::Progress)
        }
    }
}

/// Every live shortcut. Framework-neutral: the adapters own one per page and
/// feed it key presses. `H` is whatever handler type the adapter stores.
///
/// ```
/// use std::time::Duration;
/// use hydroxy_hotkeys::{HotkeyOptions, KeyEvent, KeyPress, Platform, Registry};
///
/// let mut registry = Registry::new(Platform::Mac);
/// registry.register("Mod+K".parse().unwrap(), HotkeyOptions::new().description("Search"), "search");
///
/// let event = KeyEvent { press: KeyPress { key: "k".into(), meta: true, ..KeyPress::default() }, ..KeyEvent::default() };
/// let hit = registry.dispatch(&event, Duration::ZERO, |_| None).unwrap();
/// assert_eq!(hit.handler, Some("search"));
/// assert!(hit.prevent_default);
/// ```
#[derive(Debug)]
pub struct Registry<H> {
    platform: Platform,
    entries: Vec<Entry<H>>,
    inactive_scopes: HashSet<String>,
    next_id: u64,
}

impl<H> Default for Registry<H> {
    fn default() -> Self {
        Self::new(Platform::default())
    }
}

impl<H> Registry<H> {
    /// An empty registry for `platform`.
    #[must_use]
    pub fn new(platform: Platform) -> Self {
        Self { platform, entries: Vec::new(), inactive_scopes: HashSet::new(), next_id: 0 }
    }

    /// The platform `Mod` resolves for.
    #[must_use]
    pub fn platform(&self) -> Platform {
        self.platform
    }

    /// Change the platform, once it is known.
    pub fn set_platform(&mut self, platform: Platform) {
        self.platform = platform;
    }

    /// Add a shortcut. Among equals, the one registered first wins.
    pub fn register(&mut self, hotkey: Hotkey, options: HotkeyOptions, handler: H) -> HotkeyId {
        let id = HotkeyId(self.next_id);
        self.next_id += 1;
        self.entries.push(Entry { id, hotkey, options, handler, progress: 0, last: Duration::ZERO });
        id
    }

    /// Remove a shortcut. False if it was already gone.
    pub fn unregister(&mut self, id: HotkeyId) -> bool {
        let before = self.entries.len();
        self.entries.retain(|e| e.id != id);
        self.entries.len() != before
    }

    /// Turn one shortcut on or off.
    pub fn set_enabled(&mut self, id: HotkeyId, enabled: bool) {
        if let Some(e) = self.entries.iter_mut().find(|e| e.id == id) {
            e.options.enabled = enabled;
            e.progress = 0;
        }
    }

    /// Turn a named scope on or off. Scopes are active until turned off.
    pub fn set_scope_active(&mut self, scope: &str, active: bool) {
        if active {
            self.inactive_scopes.remove(scope);
        } else {
            self.inactive_scopes.insert(scope.to_string());
        }
    }

    fn active(&self, e: &Entry<H>) -> bool {
        e.options.enabled && e.options.scope.as_ref().is_none_or(|s| !self.inactive_scopes.contains(s))
    }

    /// Every shortcut, in registration order.
    #[must_use]
    pub fn list(&self) -> Vec<HotkeyInfo> {
        self.entries
            .iter()
            .map(|e| HotkeyInfo {
                id: e.id,
                hotkey: e.hotkey.clone(),
                description: e.options.description.clone(),
                group: e.options.group.clone(),
                scope: e.options.scope.clone(),
                active: self.active(e),
            })
            .collect()
    }

    /// Pairs that clash: same scope and `within`, and one shortcut equals or
    /// starts the other (so `G` swallows `G I`). The first of each pair wins.
    #[must_use]
    pub fn conflicts(&self) -> Vec<(HotkeyId, HotkeyId)> {
        let mut out = Vec::new();
        for (i, a) in self.entries.iter().enumerate() {
            for b in &self.entries[i + 1..] {
                let same_place = a.options.scope == b.options.scope && a.options.within == b.options.within;
                if same_place
                    && (a.hotkey.starts_with(&b.hotkey, self.platform)
                        || b.hotkey.starts_with(&a.hotkey, self.platform))
                {
                    out.push((a.id, b.id));
                }
            }
        }
        out
    }

    /// The chords that could match the next key press, for active shortcuts that prevent the default.
    #[must_use]
    pub fn armed(&self) -> Vec<Armed> {
        let mut out = Vec::new();
        for e in self.entries.iter().filter(|e| e.options.prevent_default && self.active(e)) {
            let chords = e.hotkey.chords();
            let next = (e.progress > 0).then(|| &chords[e.progress]);
            for chord in std::iter::once(&chords[0]).chain(next) {
                out.push(Armed {
                    chord: chord.resolve(self.platform),
                    in_inputs: e.options.in_inputs,
                    within: e.options.within.clone(),
                });
            }
        }
        out
    }

    /// Every distinct `within` selector, so the adapter knows what to measure.
    #[must_use]
    pub fn selectors(&self) -> Vec<String> {
        let mut out: Vec<String> = self.entries.iter().filter_map(|e| e.options.within.clone()).collect();
        out.sort();
        out.dedup();
        out
    }

    /// Feed a key going down. `now` is any monotonic clock (it only times
    /// sequences). `within(selector)` says how many elements up from the focused
    /// one the nearest match is, or `None` if focus is not inside one.
    ///
    /// Every matching sequence advances; one registration wins: the innermost
    /// `within`, then the highest priority, then the earliest registered.
    pub fn dispatch(
        &mut self,
        event: &KeyEvent,
        now: Duration,
        within: impl Fn(&str) -> Option<usize>,
    ) -> Option<Dispatch<H>>
    where
        H: Clone,
    {
        let key = event.press.key.as_str();
        if is_modifier_key(key) || key == "Dead" || key == "Unidentified" {
            return None;
        }
        let platform = self.platform;
        let mut best: Option<((usize, i32, usize), Step)> = None;
        for i in 0..self.entries.len() {
            let e = &self.entries[i];
            if !self.active(e) || (event.repeat && !e.options.repeat) || (event.editable && !e.options.in_inputs) {
                continue;
            }
            let depth = match &e.options.within {
                None => usize::MAX,
                Some(selector) => match within(selector) {
                    Some(d) => d,
                    None => continue,
                },
            };
            let rank = (depth, -e.options.priority, i);
            let Some(step) = self.entries[i].advance(&event.press, now, platform) else { continue };
            if best.is_none_or(|(r, _)| rank < r) {
                best = Some((rank, step));
            }
        }
        let ((_, _, i), step) = best?;
        let e = &self.entries[i];
        Some(Dispatch {
            id: e.id,
            handler: (step == Step::Complete).then(|| e.handler.clone()),
            prevent_default: e.options.prevent_default,
            stop_propagation: e.options.stop_propagation,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ev(key: &str, mods: &str) -> KeyEvent {
        KeyEvent {
            press: KeyPress {
                key: key.into(),
                ctrl: mods.contains('c'),
                alt: mods.contains('a'),
                shift: mods.contains('s'),
                meta: mods.contains('m'),
                ..KeyPress::default()
            },
            repeat: mods.contains('r'),
            editable: mods.contains('e'),
        }
    }

    fn ms(n: u64) -> Duration {
        Duration::from_millis(n)
    }

    fn reg(r: &mut Registry<&'static str>, keys: &str, options: HotkeyOptions, name: &'static str) -> HotkeyId {
        r.register(keys.parse().unwrap(), options, name)
    }

    fn fire(r: &mut Registry<&'static str>, e: &KeyEvent, now: u64) -> Option<&'static str> {
        r.dispatch(e, ms(now), |_| None).and_then(|d| d.handler)
    }

    #[test]
    fn inputs_repeat_and_enabled() {
        let mut r = Registry::new(Platform::Other);
        let save = reg(&mut r, "Mod+S", HotkeyOptions::new().in_inputs(true), "save");
        reg(&mut r, "J", HotkeyOptions::new(), "next");
        assert_eq!(fire(&mut r, &ev("j", "e"), 0), None, "typing j in an input");
        assert_eq!(fire(&mut r, &ev("s", "ce"), 0), Some("save"), "opted in");
        assert_eq!(fire(&mut r, &ev("j", "r"), 0), None, "auto-repeat");
        r.set_enabled(save, false);
        assert_eq!(fire(&mut r, &ev("s", "c"), 0), None);
        assert!(!r.list()[0].active);
        assert_eq!(fire(&mut r, &ev("Control", "c"), 0), None);
    }

    #[test]
    fn sequences_advance_and_time_out() {
        let mut r = Registry::new(Platform::Other);
        reg(&mut r, "g i", HotkeyOptions::new(), "inbox");
        let progress = r.dispatch(&ev("g", ""), ms(0), |_| None).unwrap();
        assert_eq!(progress.handler, None);
        assert!(progress.prevent_default);
        assert_eq!(fire(&mut r, &ev("i", ""), 500), Some("inbox"));
        // Too slow.
        fire(&mut r, &ev("g", ""), 1000);
        assert_eq!(fire(&mut r, &ev("i", ""), 2500), None);
        // A wrong key resets, but can start again.
        fire(&mut r, &ev("g", ""), 3000);
        fire(&mut r, &ev("x", ""), 3100);
        assert_eq!(fire(&mut r, &ev("i", ""), 3200), None);
        fire(&mut r, &ev("g", ""), 4000);
        fire(&mut r, &ev("g", ""), 4100);
        assert_eq!(fire(&mut r, &ev("i", ""), 4200), Some("inbox"));
        // Shift held for nothing does not break a sequence.
        fire(&mut r, &ev("g", ""), 5000);
        fire(&mut r, &ev("Shift", "s"), 5050);
        assert_eq!(fire(&mut r, &ev("i", ""), 5100), Some("inbox"));
    }

    #[test]
    fn scopes_within_and_priority() {
        let mut r = Registry::new(Platform::Other);
        reg(&mut r, "Escape", HotkeyOptions::new(), "global");
        reg(&mut r, "Escape", HotkeyOptions::new().within("#outer"), "outer");
        reg(&mut r, "Escape", HotkeyOptions::new().within("#inner"), "inner");
        reg(&mut r, "Escape", HotkeyOptions::new().priority(5), "loud");
        reg(&mut r, "Escape", HotkeyOptions::new().priority(9).scope("modal"), "modal");
        let esc = ev("Escape", "");
        // Focus outside both elements: priority decides, then registration order.
        assert_eq!(fire(&mut r, &esc, 0), Some("modal"));
        r.set_scope_active("modal", false);
        assert_eq!(fire(&mut r, &esc, 0), Some("loud"));
        // Focus inside #inner, which sits inside #outer: innermost wins.
        let inside = |s: &str| match s {
            "#inner" => Some(1),
            "#outer" => Some(3),
            _ => None,
        };
        assert_eq!(r.dispatch(&esc, ms(0), inside).unwrap().handler, Some("inner"));
        let outer_only = |s: &str| (s == "#outer").then_some(0);
        assert_eq!(r.dispatch(&esc, ms(0), outer_only).unwrap().handler, Some("outer"));
        assert_eq!(r.selectors(), ["#inner", "#outer"]);
    }

    #[test]
    fn equal_rank_keeps_the_first_and_conflicts_are_listed() {
        let mut r = Registry::new(Platform::Other);
        let a = reg(&mut r, "Mod+K", HotkeyOptions::new(), "first");
        let b = reg(&mut r, "Ctrl+K", HotkeyOptions::new(), "second");
        let g = reg(&mut r, "G", HotkeyOptions::new(), "g");
        let gi = reg(&mut r, "G I", HotkeyOptions::new(), "gi");
        reg(&mut r, "G I", HotkeyOptions::new().scope("other"), "elsewhere");
        assert_eq!(fire(&mut r, &ev("k", "c"), 0), Some("first"));
        assert_eq!(r.conflicts(), [(a, b), (g, gi)]);
        assert!(r.unregister(a));
        assert!(!r.unregister(a));
        assert_eq!(fire(&mut r, &ev("k", "c"), 0), Some("second"));
    }

    #[test]
    fn armed_lists_next_chords() {
        let mut r = Registry::new(Platform::Mac);
        reg(&mut r, "Mod+S", HotkeyOptions::new(), "save");
        reg(&mut r, "g i", HotkeyOptions::new(), "inbox");
        reg(&mut r, "x", HotkeyOptions::new().prevent_default(false), "quiet");
        let names = |r: &Registry<_>| r.armed().iter().map(|a| a.chord.to_string()).collect::<Vec<_>>();
        assert_eq!(names(&r), ["Meta+S", "G"]);
        fire(&mut r, &ev("g", ""), 0);
        assert_eq!(names(&r), ["Meta+S", "G", "I"]);
    }
}
