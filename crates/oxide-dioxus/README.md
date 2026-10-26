# oxide-dioxus

Accessible UI components for Dioxus (web, desktop, mobile), built on oxide state machines and headless connectors.

Part of oxide: accessible UI components for Leptos and Dioxus. Behaviour lives in tested state machines (`oxide-machines`), a headless layer turns it into ARIA and keyboard support (`oxide-headless`), and thin adapters render it (`oxide-leptos`, `oxide-dioxus`), styled by `oxide-tokens`.

## Use

```sh
cargo add oxide-dioxus oxide-tokens
```

```rust
use dioxus::prelude::*;
use oxide_dioxus::switch::Switch;

#[component]
fn App() -> Element {
    rsx! {
        style { {oxide_tokens::CSS} }
        Switch { default_checked: true, "Email me when it's ready" }
    }
}
```

## License

MIT
