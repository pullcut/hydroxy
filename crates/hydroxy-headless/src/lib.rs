//! Connectors: turn machine state into ARIA/data attributes and map keys to
//! events. Plain data, no DOM, so every framework adapter shares it.

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
pub mod approval;
pub mod assistant_composer;
pub mod assistant_mcp;
pub mod assistant_thread;
pub mod async_validated_input;
pub mod attachment;
pub mod attrs;
pub mod audio_player;
pub mod audio_waveform;
pub mod availability_picker;
pub mod badge;
pub mod breadcrumb;
pub mod button;
pub mod calendar;
pub mod callout;
pub mod carousel;
pub mod check_list;
pub mod checkbox;
pub mod citation_popover;
pub mod code;
pub mod collapsible;
pub mod color_picker;
pub mod combobox;
pub mod command_palette;
pub mod comment_thread;
pub mod context_menu;
pub mod cron_builder;
pub mod dashboard_layout;
pub mod data_table;
pub mod date_field;
pub mod date_picker;
pub mod date_range_picker;
pub mod dialog;
pub mod diff_viewer;
pub mod dock_layout;
pub mod due;
pub mod duration_field;
pub mod emoji_picker;
pub mod file_dropzone;
pub mod file_manager;
pub mod filter_builder;
pub mod gantt;
pub mod grid;
pub mod hover_card;
pub mod icons;
pub mod inline_edit;
pub mod input;
pub mod jewel;
pub mod json_inspector;
pub mod kanban;
pub mod lightbox;
pub mod live_cursors;
pub mod log_stream;
pub mod markdown;
pub mod markdown_editor;
pub mod master_detail;
pub mod media_gallery;
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
pub mod overlay;
pub mod pagination;
pub mod password_field;
pub mod phone_field;
pub mod popover;
pub mod presence_bar;
pub mod progress_bar;
pub mod qr_code;
pub mod quantity_stepper;
pub mod questionnaire;
pub mod radio_group;
pub mod range_slider;
pub mod rating;
pub mod reaction_bar;
pub mod recording_control;
pub mod resizable;
pub mod reveal;
pub mod saved_views;
pub mod select;
pub mod sheet;
pub mod shortcut_recorder;
pub mod sidebar_layout;
pub mod skeleton;
pub mod sortable_list;
pub mod spreadsheet_grid;
pub mod switch;
pub mod table_of_contents;
pub mod tabs;
pub mod tags_input;
pub mod terminal;
pub mod theme_menu;
pub mod thread;
pub mod thread_list;
pub mod toast;
pub mod toggle_button;
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
pub mod image_cropper;
pub mod node_editor;
pub mod signature_pad;
pub mod theme_editor;
pub mod whiteboard;
