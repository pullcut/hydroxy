//! Connectors: turn machine state into ARIA/data attributes and map keys to
//! events. Plain data, no DOM, so every framework adapter shares it.

#![cfg_attr(
    test,
    allow(clippy::assert_is_empty, reason = "`assert!(x.is_empty())` reads better in tests than `assert_eq!(x, [] as [T; 0])`")
)]

pub mod access_editor;
pub mod availability_picker;
pub mod media_timeline;
pub mod questionnaire;
pub mod voice_input;
pub mod app_shell;
pub mod audio_player;
pub mod audio_waveform;
pub mod master_detail;
pub mod recording_control;
pub mod table_of_contents;
pub mod transcript_viewer;
pub mod voice_session;
pub mod accordion;
pub mod comment_thread;
pub mod log_stream;
pub mod message_scroller;
pub mod thread;
pub mod thread_list;
pub mod assistant_composer;
pub mod assistant_mcp;
pub mod assistant_thread;
pub mod approval;
pub mod kanban;
pub mod inline_edit;
pub mod async_validated_input;
pub mod shortcut_recorder;
pub mod sortable_list;
pub mod filter_builder;
pub mod tree_view;
pub mod button;
pub mod calendar;
pub mod date_picker;
pub mod color_picker;
pub mod date_field;
pub mod date_range_picker;
pub mod duration_field;
pub mod quantity_stepper;
pub mod check_list;
pub mod checkbox;
pub mod code;
pub mod collapsible;
pub mod dashboard_layout;
pub mod dock_layout;
pub mod grid;
pub mod carousel;
pub mod reveal;
pub mod data_table;
pub mod resizable;
pub mod sidebar_layout;
pub mod context_menu;
pub mod hover_card;
pub mod pagination;
pub mod sheet;
pub mod combobox;
pub mod command_palette;
pub mod dialog;
pub mod due;
pub mod icons;
pub mod input;
pub mod jewel;
pub mod markdown;
pub mod markdown_editor;
pub mod menu;
pub mod number_field;
pub mod otp_input;
pub mod overlay;
pub mod popover;
pub mod progress_bar;
pub mod radio_group;
pub mod range_slider;
pub mod rating;
pub mod toggle_button;
pub mod file_dropzone;
pub mod wizard;
pub mod select;
pub mod switch;
pub mod tabs;
pub mod tags_input;
pub mod toast;
pub mod tooltip;
pub mod upload_queue;
pub mod attachment;
pub mod saved_views;
pub mod file_manager;
pub mod gantt;
pub mod menubar;
pub mod navigation_menu;
pub mod notification_center;
pub mod theme_menu;

use std::sync::atomic::{AtomicUsize, Ordering};

/// A unique id prefix per component instance.
// process-wide counter, fine for CSR; SSR+hydration needs ids from the framework.
#[must_use]
pub fn next_id(scope: &str) -> String {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    format!("ox-{scope}-{}", NEXT.fetch_add(1, Ordering::Relaxed))
}

/// Layout axis. Decides which arrow keys move focus.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Orientation {
    /// Left and Right arrows.
    #[default]
    Horizontal,
    /// Up and Down arrows.
    Vertical,
}

impl Orientation {
    /// The `aria-orientation` / `data-orientation` value.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Orientation::Horizontal => "horizontal",
            Orientation::Vertical => "vertical",
        }
    }
}
