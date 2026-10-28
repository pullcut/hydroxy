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
pub use nav::{Nav, first_option, last_option, navigate, option_from};
pub use sections::{Block, blocks, header_for, is_header, next_header, option_count, pos_in_set, sticky_inset};
