# oxide-hotkeys

Keyboard shortcuts for oxide. Write them as strings, register them from a component, list them for a cheat sheet.

```rust
use oxide_hotkeys::{Hotkey, Platform};

let search: Hotkey = "Mod+K".parse().unwrap();
assert_eq!(search.label(Platform::Mac), "⌘K");
assert_eq!(search.label(Platform::Other), "Ctrl+K");
```

- `Mod` is ⌘ on a Mac and Ctrl elsewhere; `Ctrl`, `Meta`, `Alt`, `Shift` are literal. Aliases like `cmd`, `option`, `esc`, `up` work.
- Keys match what the layout prints (`KeyboardEvent.key`). Write `code:KeyK` to match the physical key instead.
- Spaces make a sequence: `g i` is G, then I within a second.
- Shortcuts skip typing in inputs, textareas, selects and contenteditable unless `in_inputs(true)`.
- `scope("modal")` groups shortcuts you switch on and off together; `within("#editor")` fires only while focus is inside that element, and the innermost match wins. Then higher `priority`, then whoever registered first.
- `preventDefault` is on by default.
- `Registry::list` gives every shortcut with its description and group; `Registry::conflicts` flags clashes, including `G` swallowing `G I`.

The core has no dependencies and takes an explicit clock, so it is tested without a browser. The shortcut recorder in `oxide-machines` uses the same key naming, so a recorded `Mod+Shift+K` parses as-is.

## Leptos and Dioxus

Turn on the `leptos` or `dioxus` feature:

```rust,ignore
use oxide_hotkeys::HotkeyOptions;
use oxide_hotkeys::leptos::{use_hotkey, use_hotkey_with, use_hotkeys};

use_hotkey("Mod+S", move || save());
use_hotkey_with("g i", HotkeyOptions::new().description("Go to inbox").group("Go to"), move || go("/inbox"));
let all = use_hotkeys(); // Signal<Vec<HotkeyInfo>> for a cheat sheet
```

Shortcuts unregister when the component unmounts. On the server the hooks do nothing. The Dioxus hooks go through `document::eval`, so they work on web, desktop and mobile; there `stop_propagation` is not supported.

## License

MIT
