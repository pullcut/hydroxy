# oxide-leptos

Accessible UI components for Leptos, built on oxide state machines and headless connectors. Works with server rendering.

Part of oxide: accessible UI components for Leptos and Dioxus. Behaviour lives in tested state machines (`oxide-machines`), a headless layer turns it into ARIA and keyboard support (`oxide-headless`), and thin adapters render it (`oxide-leptos`, `oxide-dioxus`), styled by `oxide-tokens`.

## Use

```sh
cargo add oxide-leptos oxide-tokens
```

```rust
use leptos::prelude::*;
use oxide_leptos::switch::Switch;

#[component]
fn App() -> impl IntoView {
    view! {
        <style inner_html=oxide_tokens::CSS></style>
        <Switch default_checked=true>"Email me when it's ready"</Switch>
    }
}
```

## License

MIT
