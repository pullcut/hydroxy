use hydroxy_hotkeys::leptos::{hotkey_platform, use_hotkey_with, use_hotkeys};
use hydroxy_hotkeys::{HotkeyInfo, HotkeyOptions};
use hydroxy_leptos::input::Input;
use hydroxy_leptos::kbd::Kbd;
use leptos::prelude::*;

fn about(text: &str, group: &str) -> HotkeyOptions {
    HotkeyOptions::new().description(text).group(group)
}

pub fn demo() -> AnyView {
    let (last, set_last) = signal(String::from("Nothing yet"));
    let (count, set_count) = signal(0i32);
    let (sheet, set_sheet) = signal(true);
    let said = move |text: &'static str| move || set_last.set(text.to_string());

    use_hotkey_with("Mod+Shift+U", about("Say hello", "General"), said("Hello from Mod+Shift+U"));
    use_hotkey_with("?", about("Show or hide this list", "General"), move || set_sheet.update(|s| *s = !*s));
    // Escape opts in, so it works while typing too.
    use_hotkey_with("Escape", about("Reset the counter", "General").in_inputs(true), move || set_count.set(0));
    // Sequences: press G, let go, then I within a second.
    use_hotkey_with("g i", about("Go to inbox", "Go to"), said("Went to the inbox"));
    use_hotkey_with("g s", about("Go to settings", "Go to"), said("Went to settings"));
    use_hotkey_with("j", about("Count up", "Counter"), move || set_count.update(|n| *n += 1));
    use_hotkey_with("k", about("Count down", "Counter"), move || set_count.update(|n| *n -= 1));
    // Only while focus is inside the pad.
    use_hotkey_with(
        "n",
        about("Note (inside the pad only)", "Pad").within("#hotkeys-pad"),
        said("Note taken in the pad"),
    );

    let box_ = "display: grid; gap: var(--space-3); padding: var(--space-5); border-radius: var(--radius-3); background: var(--surface-sunken);";
    let line = "margin: 0; font: var(--text-sm); color: var(--text-2);";

    view! {
        <div style="display: grid; gap: var(--space-6); max-width: 560px;">
            <section style=box_>
                <p style=line>"Last: " {last}</p>
                <p style=line>"Counter: " {count} " (J and K, Escape resets)"</p>
                <Input label="Typing here is ignored" placeholder="j and k do nothing in here" />
                <div
                    id="hotkeys-pad"
                    tabindex="0"
                    style="padding: var(--space-4); border: 1px dashed var(--border-2); border-radius: var(--radius-2); font: var(--text-sm); color: var(--text-2);"
                >
                    "Click here, then press N"
                </div>
            </section>
            <Show when=move || sheet.get()>
                <HotkeyCheatSheet />
            </Show>
        </div>
    }
    .into_any()
}

/// Every live shortcut, grouped. A demo helper, not an hydroxy component.
#[component]
fn HotkeyCheatSheet() -> impl IntoView {
    let hotkeys = use_hotkeys();
    let groups = move || {
        let mut out: Vec<(String, Vec<HotkeyInfo>)> = Vec::new();
        for info in hotkeys.get().into_iter().filter(|h| h.active && h.description.is_some()) {
            let group = info.group.clone().unwrap_or_default();
            match out.iter_mut().find(|(g, _)| *g == group) {
                Some((_, list)) => list.push(info),
                None => out.push((group, vec![info])),
            }
        }
        out
    };
    view! {
        <section style="display: grid; gap: var(--space-4);" aria-label="Keyboard shortcuts">
            {move || {
                let platform = hotkey_platform();
                groups()
                    .into_iter()
                    .map(|(group, list)| {
                        view! {
                            <div style="display: grid; gap: var(--space-2);">
                                <h3 style="margin: 0; font: var(--text-sm); font-weight: 600;">{group}</h3>
                                <dl style="margin: 0; display: grid; grid-template-columns: 1fr auto; gap: var(--space-2) var(--space-4); font: var(--text-sm); color: var(--text-2);">
                                    {list
                                        .into_iter()
                                        .map(|info| {
                                            let chords = info.hotkey.keys(platform);
                                            let last = chords.len() - 1;
                                            view! {
                                                <dt>{info.description}</dt>
                                                <dd style="margin: 0; display: flex; gap: var(--space-1); align-items: center;">
                                                    {chords
                                                        .into_iter()
                                                        .enumerate()
                                                        .map(|(i, caps)| {
                                                            view! {
                                                                {caps.into_iter().map(|cap| view! { <Kbd>{cap}</Kbd> }).collect_view()}
                                                                {(i < last).then_some(" then ")}
                                                            }
                                                        })
                                                        .collect_view()}
                                                </dd>
                                            }
                                        })
                                        .collect_view()}
                                </dl>
                            </div>
                        }
                    })
                    .collect_view()
            }}
        </section>
    }
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "hotkeys",
    name: "Hotkeys",
    group: "Utilities",
    summary: "Keyboard shortcuts from strings like Mod+K (⌘ on a Mac, Ctrl elsewhere) and sequences like G I. Scopes, focus-bound shortcuts, priorities, typing in inputs ignored by default, and a registry that lists everything for a cheat sheet.",
    demo,
    leptos: include_str!("hotkeys.rs"),
    dioxus: include_str!("../../../../crates/hydroxy-hotkeys/examples/dioxus.rs"),
    parts: &[],
    keys: &[
        &["Mod+Shift+U", "Say hello"],
        &["?", "Show or hide the cheat sheet"],
        &["G then I / G then S", "Sequences"],
        &["J / K", "Count up and down, ignored while typing"],
        &["Escape", "Reset the counter, even while typing"],
        &["N", "Only while focus is in the pad"],
    ],
    props: &[
        &[
            "use_hotkey",
            "(&str, Fn()) -> HotkeyHandle",
            "Registers until the component unmounts. Panics on an invalid shortcut",
        ],
        &[
            "use_hotkey_with",
            "(&str, HotkeyOptions, Fn()) -> HotkeyHandle",
            "With options. handle.set_enabled(bool) turns it on and off",
        ],
        &[
            "HotkeyOptions",
            "description, group, scope, within, enabled, prevent_default, stop_propagation, in_inputs, repeat, priority, timeout",
            "Builder. prevent_default is on by default; in_inputs, repeat off; timeout 1s",
        ],
        &[
            "use_hotkeys",
            "() -> Signal<Vec<HotkeyInfo>>",
            "Every registered shortcut for a cheat sheet. Empty until mounted",
        ],
        &["set_hotkey_scope", "(&str, bool)", "Turn a named scope on or off"],
        &[
            "Hotkey",
            "FromStr, keys(Platform), label(Platform)",
            "Mod Ctrl Meta Alt Shift + key; code:KeyK for a physical key; spaces separate sequence steps",
        ],
        &["Registry", "register, unregister, dispatch, list, conflicts", "The framework-neutral core the hooks run on"],
    ],
    events: &[],
};
