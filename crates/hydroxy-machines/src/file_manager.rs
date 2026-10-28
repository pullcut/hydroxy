//! File manager owns which folder is open, the selection, the focused item,
//! the rename in progress and the list/grid choice. The items themselves
//! belong to the owner: renames and deletes are reported as commands, and the
//! owner applies them (see [`apply_rename`] and [`apply_delete`]) and passes
//! the new list back in.

mod items;
mod machine;
mod model;
#[cfg(test)]
mod tests;

pub use items::*;
pub use machine::FileManagerMachine;
pub use model::*;
