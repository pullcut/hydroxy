# hydroxy-leptos

Accessible UI components for Leptos, built on hydroxy state machines and headless connectors. Works with server rendering.

Part of hydroxy: accessible UI components for Leptos and Dioxus. Behaviour lives in tested state machines (`hydroxy-machines`), a headless layer turns it into ARIA and keyboard support (`hydroxy-headless`), and thin adapters render it (`hydroxy-leptos`, `hydroxy-dioxus`), styled by `hydroxy-tokens`.

## Use

```sh
cargo add hydroxy-leptos hydroxy-tokens
```

```rust
use leptos::prelude::*;
use hydroxy_leptos::switch::Switch;

#[component]
fn App() -> impl IntoView {
    view! {
        <style inner_html=hydroxy_tokens::CSS></style>
        <Switch default_checked=true>"Email me when it's ready"</Switch>
    }
}
```

## License

MIT
