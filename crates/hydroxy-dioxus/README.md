# hydroxy-dioxus

Accessible UI components for Dioxus (web, desktop, mobile), built on hydroxy state machines and headless connectors.

Part of hydroxy: accessible UI components for Leptos and Dioxus. Behaviour lives in tested state machines (`hydroxy-machines`), a headless layer turns it into ARIA and keyboard support (`hydroxy-headless`), and thin adapters render it (`hydroxy-leptos`, `hydroxy-dioxus`), styled by `hydroxy-tokens`.

## Use

```sh
cargo add hydroxy-dioxus hydroxy-tokens
```

```rust
use dioxus::prelude::*;
use hydroxy_dioxus::switch::Switch;

#[component]
fn App() -> Element {
    rsx! {
        style { {hydroxy_tokens::CSS} }
        Switch { default_checked: true, "Email me when it's ready" }
    }
}
```

## License

MIT
