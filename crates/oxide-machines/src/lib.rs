//! Framework-neutral component behaviour. One module per component.

pub mod accordion;
pub mod checkbox;
pub mod collapsible;
pub mod combobox;
pub mod command_palette;
pub mod dialog;
pub mod list;
pub mod listbox;
pub mod menu;
pub mod popover;
pub mod radio_group;
pub mod switch;
pub mod tabs;
pub mod tags_input;
pub mod toast;
pub mod tooltip;

/// Why a value changed. Reported with every component's change command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeReason {
    /// Click or tap. Also Space or Enter when they arrive as a native button click.
    Pointer,
    /// Arrow, Home or End key.
    Keyboard,
    /// The owner asked for it in code.
    Programmatic,
}
