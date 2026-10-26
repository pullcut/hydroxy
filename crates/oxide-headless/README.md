# oxide-headless

Headless UI connectors: ARIA attributes, roving focus, key mapping and overlay placement as plain data, shared by oxide adapters.

Part of oxide: accessible UI components for Leptos and Dioxus. Behaviour lives in tested state machines (`oxide-machines`), a headless layer turns it into ARIA and keyboard support (`oxide-headless`), and thin adapters render it (`oxide-leptos`, `oxide-dioxus`), styled by `oxide-tokens`.

## Use

Most apps use it through `oxide-leptos` or `oxide-dioxus` rather than directly.

## License

MIT
