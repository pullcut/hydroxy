# Changelog

All notable changes to this project are recorded here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the project follows
[Semantic Versioning](https://semver.org/).

## [Unreleased]

### Added

- 36 new charts and 30 new components, each with a Leptos and a Dioxus version.
- New crates: `hydroxy-hotkeys`, `hydroxy-pacer`, `hydroxy-virtual` and `hydroxy-state`.
- `hydroxy_tokens::css`: one constant per stylesheet plus `css::BASE`, so apps can ship only
  the styles they use. `CSS` is still the whole bundle.
- `hydroxy_headless::attrs` (`data_state`, `roving`, `nav_key`, `step_key`, `printable_char`),
  shared id helpers and value-text helpers.
- `hydroxy_statechart::step` and `Controllable<T>`.
- Shared chart helpers: `geom`, `scale::extent`, `plot::pct`, `line::windowed` and more.
- Criterion benchmarks for virtual list layout, `line::decimate`, `raster::scatter`, Venn and
  Voronoi layout, and spreadsheet recalculation.
- Playwright interaction and axe accessibility specs for the docs site.
- `deny.toml` for `cargo deny`.

### Changed

- **Breaking:** `Transition.state` is now `Option<S>` and `sync` returns `Option<State>`, so a machine can
  say "no change" without cloning its state.
- **Breaking:** Moved `Tone` to `hydroxy_headless::badge` and `CalloutKind` to `hydroxy_headless::callout`.
  Both adapters re-export them from their own `badge` and `callout` modules.
- **Breaking:** The state structs of accordion, assistant_thread (`BranchPickerState`),
  async_validated_input, access_editor, availability_picker, calendar, carousel, checkbox,
  collapsible, combobox, command_palette, comment_thread, dashboard_layout, data_table,
  date_picker, date_range_picker, dialog, dock_layout, filter_builder, gantt, grid, hover_card,
  inline_edit, kanban and log_stream now hold a `Controllable<T>` in place of the value field
  plus `controlled: bool`. Read the value as `state.field.value` and the flag as
  `state.field.controlled`. Gantt's `controlled_scale` is now `scale.controlled`.
- The docs page list and the Dioxus gallery are each declared once with a macro.
- `js-sys` is a workspace dependency.
- docs.rs builds `hydroxy-hotkeys`, `hydroxy-state` and `hydroxy-pacer` with all features, so
  their Leptos and Dioxus hooks appear.

### Removed

- **Breaking:** the duplicate `Orientation` and `Role` types are gone. `toolbar::Orientation` is now
  `hydroxy_headless::Orientation` and `assistant_thread::Role` is now
  `hydroxy_headless::thread::Role`; the old paths re-export the single definition.

### Fixed

- Charts, pacer and machines no longer panic or hang on extreme input (NaN, infinity, empty or
  huge data).
- Leptos components release the microphone on unmount and drop their closures, listeners and
  observers.
- Citation links are sanitised, and strings handed to JavaScript are escaped properly.
- Machines report bad input with typed errors, and their enums parse with `FromStr`.
- Transitions that change nothing no longer trigger a re-render, and large state is shared
  rather than cloned.
- CI passes again: the desktop gallery is excluded, and the MSRV, fmt and clippy jobs are fixed.

[Unreleased]: https://github.com/pullcut/hydroxy/commits/main
