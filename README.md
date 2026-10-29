# hydroxy

Headless UI components for Leptos and Dioxus.

## Size

`examples/size-probe` is the smallest useful app: one `Button` and one `LineChart`, with
only the CSS they use. Built with the `wasm-release` profile and run through wasm-bindgen,
before wasm-opt:

| | bytes |
|---|---|
| `size-probe_bg.wasm` | 407,327 |
| gzip -9 | 138,818 |

CI fails if the `.wasm` grows more than 10% past `examples/size-probe/wasm-size-budget.txt`.
