//! The docs demo in Dioxus: shortcuts, a sequence, a focus-bound key and a cheat sheet.

use dioxus::prelude::*;
use oxide_hotkeys::HotkeyOptions;
use oxide_hotkeys::dioxus::{hotkey_platform, use_hotkey_with, use_hotkeys};

fn about(text: &str, group: &str) -> HotkeyOptions {
    HotkeyOptions::new().description(text).group(group)
}

#[component]
fn HotkeysDemo() -> Element {
    let mut last = use_signal(|| String::from("Nothing yet"));
    let mut count = use_signal(|| 0i32);

    use_hotkey_with("Mod+Shift+U", about("Say hello", "General"), move || last.set("Hello from Mod+Shift+U".into()));
    use_hotkey_with("Escape", about("Reset the counter", "General").in_inputs(true), move || count.set(0));
    use_hotkey_with("g i", about("Go to inbox", "Go to"), move || last.set("Went to the inbox".into()));
    use_hotkey_with("j", about("Count up", "Counter"), move || count += 1);
    use_hotkey_with("k", about("Count down", "Counter"), move || count -= 1);
    use_hotkey_with("n", about("Note (inside the pad only)", "Pad").within("#hotkeys-pad"), move || {
        last.set("Note taken in the pad".into());
    });

    rsx! {
        p { "Last: {last}" }
        p { "Counter: {count}" }
        input { placeholder: "j and k do nothing in here" }
        div { id: "hotkeys-pad", tabindex: "0", "Click here, then press N" }
        HotkeyCheatSheet {}
    }
}

/// Every live shortcut with its keys. A demo helper, not an oxide component.
#[component]
fn HotkeyCheatSheet() -> Element {
    let hotkeys = use_hotkeys();
    let platform = hotkey_platform();
    rsx! {
        dl {
            for info in hotkeys().into_iter().filter(|h| h.active) {
                dt { {info.description.clone().unwrap_or_default()} }
                dd { {info.hotkey.label(platform)} }
            }
        }
    }
}

fn main() {
    let _ = HotkeysDemo;
}
