# hydroxy

Headless UI components for Leptos and Dioxus.

## Size

`examples/size-probe` is the smallest useful app: one `Button` and one `LineChart`, with
only the CSS they use, linked as plain stylesheets so none of it is compiled into the
wasm. Built with the `wasm-release` profile and run through wasm-bindgen,
before wasm-opt:

| | bytes |
|---|---|
| `size-probe_bg.wasm` | 333,448 |
| gzip -9 | 124,371 |
| CSS, gzip -9 | 13,870 |

CI fails if the `.wasm` grows more than 10% past `examples/size-probe/wasm-size-budget.txt`.
