//! Framework-neutral component behaviour. One module per component.

#![cfg_attr(
    test,
    allow(
        clippy::assert_is_empty,
        reason = "`assert!(x.is_empty())` reads better in tests than `assert_eq!(x, [] as [T; 0])`"
    )
)]

pub mod access_editor;
pub mod availability_picker;
pub mod media_timeline;
pub mod questionnaire;
pub mod app_shell;
pub mod audio_player;
pub mod audio_waveform;
pub mod master_detail;
pub mod table_of_contents;
pub mod transcript_viewer;
pub mod accordion;
pub mod calendar;
pub mod checkbox;
pub mod collapsible;
pub mod dashboard_layout;
pub mod dock_layout;
pub mod grid;
pub mod carousel;
pub mod reveal;
pub mod combobox;
pub mod command_palette;
pub mod data_table;
pub mod date;
pub mod date_picker;
pub mod color;
pub mod color_picker;
pub mod date_field;
pub mod date_range_picker;
pub mod duration_field;
pub mod segments;
pub mod dialog;
pub mod file_dropzone;
pub mod hover_card;
pub mod kanban;
pub mod inline_edit;
pub mod async_validated_input;
pub mod shortcut_recorder;
pub mod sortable_list;
pub mod filter_builder;
pub mod list;
pub mod listbox;
pub mod log_stream;
pub mod menu;
pub mod number_field;
pub mod otp_input;
pub mod popover;
pub mod radio_group;
pub mod range_slider;
pub mod rating;
pub mod resizable;
pub mod switch;
pub mod tabs;
pub mod tags_input;
pub mod toast;
pub mod tooltip;
pub mod menubar;
pub mod navigation_menu;
pub mod notification_center;
pub mod tree_view;
pub mod upload_queue;
pub mod saved_views;
pub mod file_manager;
pub mod gantt;
pub mod wizard;

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
