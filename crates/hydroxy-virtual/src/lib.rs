//! Windowing for very long lists: which rows to render for a scroll position, where
//! each one sits, and listbox keyboard movement over the rows that are options.
//!
//! [`Layout`] is the framework-neutral core. It holds one size per row (fixed, or
//! measured and updated as rows render) as a Fenwick tree, so offsets, point updates and
//! "which row is at this pixel" are all `O(log n)`. A list where every row has the same
//! size never allocates per row and answers in `O(1)`. Nothing here knows about
//! listboxes; grids, tables and logs can window with [`Layout`] alone.
//!
//! Group headers are rows too. They are listed by index (sorted, ascending) and are
//! skipped by keyboard movement and by `aria-posinset`. [`blocks`] groups the rendered
//! rows under their header so an adapter can give each group a box spanning its rows and
//! make the header `position: sticky` inside it: the browser then pins the header and
//! pushes it off when the next group arrives, with no script per scrolled pixel.
//!
//! ```
//! use hydroxy_virtual::Layout;
//!
//! let mut rows = Layout::fixed(10_000, 32.0);
//! // A 100px viewport scrolled to 320px shows rows 10 to 13.
//! assert_eq!(rows.visible(320.0, 100.0), 10..14);
//! // Measuring row 0 at 64px pushes every later row down.
//! assert!(rows.set_size(0, 64.0));
//! assert_eq!(rows.offset(1), 64.0);
//! ```

#![allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "row counts and pixel offsets stay far below 2^52"
)]

mod layout;
mod nav;
mod sections;

pub use layout::{Align, DEFAULT_OVERSCAN, Layout};
pub use nav::{Direction, Nav, first_option, last_option, navigate, option_from};
pub use sections::{Block, blocks, header_for, is_header, next_header, option_count, pos_in_set, sticky_inset};
