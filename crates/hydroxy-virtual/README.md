# hydroxy-virtual

Framework-free windowing for very long lists, grids and logs.

- `Layout` holds one size per row in a Fenwick tree: offsets, size updates and
  "which row is at this pixel" are `O(log n)`; uniform rows cost `O(1)` and no memory per row.
- `Layout::range` gives the rows to render for a scroll position (with overscan);
  `Layout::scroll_to` aligns a row to the start, centre or end of the viewport.
- Header helpers (`blocks`, `sticky_inset`, `navigate`, `pos_in_set`) group rows under
  sticky headers and move a listbox selection over the rows that are options.

```rust
use hydroxy_virtual::Layout;

let layout = Layout::fixed(100_000, 36.0);
let rows = layout.range(72_000.0, 600.0, 6);
assert!(rows.contains(&2_000));
```

`hydroxy-leptos` and `hydroxy-dioxus` build their `VirtualList` component on it.
