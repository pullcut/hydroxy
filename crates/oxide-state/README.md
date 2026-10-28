# oxide-state

Undo and redo for any value. `History<T>` keeps shared `Arc` snapshots of what the value
was before each change, up to a limit (100 by default), and swaps them in and out on
`undo` / `redo`. The owner keeps the current value.

Features `leptos` and `dioxus` add `use_history(initial)`, a copyable handle with
`get`, `set`, `update`, `undo`, `redo`, `can_undo` and `can_redo`.

oxide's node editor and whiteboard keep their undo stacks in a `History`.
