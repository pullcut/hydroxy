//! Framework-neutral component behaviour. One module per component.
//!
//! ```
//! use hydroxy_machines::ChangeReason;
//! use hydroxy_machines::switch::{SwitchCommand, SwitchEvent, SwitchInput, SwitchMachine};
//! use hydroxy_statechart::{Machine, step};
//!
//! let input = SwitchInput::default();
//! let on = step::<SwitchMachine>(&SwitchMachine::initial(&input), SwitchEvent::Toggle(ChangeReason::Pointer), &input);
//! assert!(on.state.checked);
//! assert_eq!(on.commands, [SwitchCommand::CheckedChange { checked: true, reason: ChangeReason::Pointer }]);
//! ```

#![cfg_attr(
    test,
    allow(
        clippy::assert_is_empty,
        reason = "`assert!(x.is_empty())` reads better in tests than `assert_eq!(x, [] as [T; 0])`"
    )
)]

pub mod access_editor;
pub mod accordion;
pub mod app_shell;
mod as_you_type;
pub mod assistant_composer;
pub mod assistant_mcp;
pub mod assistant_thread;
pub mod async_validated_input;
pub mod audio_player;
pub mod audio_waveform;
pub mod availability_picker;
pub mod breadcrumb;
pub mod calendar;
pub mod carousel;
pub mod checkbox;
pub mod citation_popover;
pub mod collapsible;
pub mod color;
pub mod color_picker;
pub mod combobox;
pub mod command_palette;
pub mod comment_thread;
pub mod cron_builder;
pub mod dashboard_layout;
pub mod data_table;
pub mod date;
pub mod date_field;
pub mod date_picker;
pub mod date_range_picker;
pub mod dialog;
pub mod diff_viewer;
pub mod dock_layout;
pub mod duration_field;
pub mod emoji_picker;
pub mod file_dropzone;
pub mod file_manager;
pub mod filter_builder;
pub mod gantt;
pub mod grid;
pub mod hover_card;
pub mod inline_edit;
pub mod json_inspector;
pub mod kanban;
pub mod lightbox;
pub mod list;
pub mod listbox;
pub mod live_cursors;
pub mod log_stream;
pub mod master_detail;
pub mod media_timeline;
pub mod mention_input;
pub mod menu;
pub mod menubar;
pub mod message_scroller;
pub mod model_picker;
pub mod money_field;
pub mod navigation_menu;
pub mod notification_center;
pub mod number_field;
pub mod otp_input;
pub mod password_field;
pub mod phone_field;
pub mod popover;
pub mod presence_bar;
pub mod qr_code;
pub mod questionnaire;
pub mod radio_group;
pub mod range_slider;
pub mod rating;
pub mod reaction_bar;
pub mod recording_control;
pub mod resizable;
pub mod reveal;
pub mod saved_views;
pub mod scroll;
pub mod segments;
pub mod shortcut_recorder;
pub mod skeleton;
pub mod sortable_list;
pub mod spreadsheet_grid;
pub mod switch;
pub mod table_of_contents;
pub mod tabs;
pub mod tags_input;
pub mod terminal;
pub mod text;
pub mod thread_list;
pub mod toast;
pub mod tool_trace;
pub mod toolbar;
pub mod tooltip;
pub mod tour;
pub mod transcript_viewer;
pub mod tree_view;
pub mod upload_queue;
pub mod virtual_list;
pub mod voice_input;
pub mod voice_session;
pub mod week_scheduler;
pub mod wizard;

/// Why a value changed. Reported with every component's change command.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChangeReason {
    /// Click or tap. Also Space or Enter when they arrive as a native button click.
    Pointer,
    /// Arrow, Home or End key.
    Keyboard,
    /// The owner asked for it in code.
    Programmatic,
}

/// Modifier keys held during a key or pointer event.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[allow(clippy::struct_excessive_bools, reason = "one flag per modifier key, like `KeyboardEvent`")]
pub struct Modifiers {
    /// Shift.
    pub shift: bool,
    /// Alt, or Option on a Mac.
    pub alt: bool,
    /// Control.
    pub ctrl: bool,
    /// Meta: Cmd on a Mac, the Windows key elsewhere.
    pub meta: bool,
}

impl Modifiers {
    /// Nothing held.
    pub const NONE: Self = Self { shift: false, alt: false, ctrl: false, meta: false };
    /// Shift alone.
    pub const SHIFT: Self = Self { shift: true, ..Self::NONE };
    /// Alt alone.
    pub const ALT: Self = Self { alt: true, ..Self::NONE };
    /// Control alone.
    pub const CTRL: Self = Self { ctrl: true, ..Self::NONE };
    /// Meta alone.
    pub const META: Self = Self { meta: true, ..Self::NONE };

    /// Ctrl or Meta: the shortcut key on any platform.
    #[must_use]
    pub const fn command(self) -> bool {
        self.ctrl || self.meta
    }
}

/// A string that names none of an enum's values, from that enum's `FromStr`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnknownName;

impl std::fmt::Display for UnknownName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("unknown name")
    }
}

impl std::error::Error for UnknownName {}
pub mod canvas;
pub mod image_cropper;
pub mod node_editor;
pub mod signature_pad;
pub mod theme_editor;
pub mod whiteboard;
