# hydroxy

Headless UI components for Leptos and Dioxus.

## Size

`examples/size-probe` is the smallest useful app: one `Button` and one `LineChart`.
Built with `cargo build -p size-probe --release --target wasm32-unknown-unknown` and
`CARGO_PROFILE_RELEASE_OPT_LEVEL=z`, before wasm-bindgen and wasm-opt:

| | bytes |
|---|---|
| `.wasm` | 1,613,569 |
| gzip -9 | 408,379 |

CI fails if the `.wasm` grows more than 10% past `examples/size-probe/wasm-size-budget.txt`.
