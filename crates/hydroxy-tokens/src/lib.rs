//! hydroxy design tokens and component styles as plain CSS.
//! Inject `CSS` once in a `<style>` tag, or copy the files from `css/`.
//!
//! Fonts are separate. The tokens name Unbounded, Hanken Grotesk and `JetBrains Mono`
//! with system fallbacks. For the quickest start, put [`GOOGLE_FONTS`] *before* `CSS`;
//! for the fastest pages, self-host the fonts with `@font-face` and preload them.

/// An `@import` of the three fonts from Google Fonts. Place it first in the stylesheet:
/// `@import` only works at the top. It blocks rendering on a second origin, so prefer
/// self-hosting on sites where first paint matters.
pub const GOOGLE_FONTS: &str = include_str!("../css/fonts.css");

/// Opt-in `@import` of the fonts the neobrutalist and mecha themes name. Without it
/// those themes fall back to the default fonts.
pub const THEME_FONTS: &str = include_str!("../css/theme-fonts.css");

/// Every token and component stylesheet, concatenated in load order. Built from the
/// constants in [`css`].
pub const CSS: &str = css::CONCAT;

/// One constant per stylesheet, so an app can ship only what it uses.
///
/// [`BASE`] (tokens, themes and base rules) is needed by every component; each other
/// constant is one component's styles. Charts need [`BASE`], [`CHART`] and their family
/// file (for example [`CHART_CARTESIAN`]). [`ALL`] lists every constant in load order and
/// the crate-level [`CSS`](crate::CSS) is exactly their concatenation.
pub mod css {
    macro_rules! sheets {
        (base: [$($bfile:literal),* $(,)?]; $($name:ident = $file:literal,)*) => {
            /// Colours, typography, spacing, effects, themes and base rules. Every
            /// component needs this.
            pub const BASE: &str = concat!($(include_str!(concat!("../css/", $bfile))),*);
            $(
                #[doc = concat!("The `", $file, "` stylesheet.")]
                pub const $name: &str = include_str!(concat!("../css/", $file));
            )*
            /// Every stylesheet in load order: [`BASE`] first, then one per file.
            pub const ALL: &[&str] = &[BASE, $($name),*];
            pub(crate) const CONCAT: &str = concat!(
                $(include_str!(concat!("../css/", $bfile)),)*
                $(include_str!(concat!("../css/", $file))),*
            );
        };
    }

    sheets! {
        base: ["colors.css", "typography.css", "spacing.css", "effects.css", "themes.css", "base.css"];
        OVERLAY = "overlay.css",
        BUTTON = "button.css",
        ICON_BUTTON = "icon-button.css",
        BADGE = "badge.css",
        TAG = "tag.css",
        SPACE_ICON = "space-icon.css",
        DOC_CARD = "doc-card.css",
        CHECK_LIST = "check-list.css",
        STEP_TRACK = "step-track.css",
        NAV_ITEM = "nav-item.css",
        NAV_SECTION = "nav-section.css",
        CHAT_BUBBLE = "chat-bubble.css",
        COMPOSER = "composer.css",
        ASSISTANT_COMPOSER = "assistant-composer.css",
        MESSAGE_SCROLLER = "message-scroller.css",
        THREAD = "thread.css",
        THREAD_LIST = "thread-list.css",
        ASSISTANT_MCP = "assistant-mcp.css",
        ASSISTANT_THREAD = "assistant-thread.css",
        ASSISTANT_THREAD_LIST = "assistant-thread-list.css",
        ASSISTANT_MODAL = "assistant-modal.css",
        COMMENT_THREAD = "comment-thread.css",
        LOG_STREAM = "log-stream.css",
        JSON_INSPECTOR = "json-inspector.css",
        TERMINAL = "terminal.css",
        INSIGHT_NOTE = "insight-note.css",
        ACTIVITY = "activity.css",
        STATUS_DIAMOND = "status-diamond.css",
        PRIORITY_MARK = "priority-mark.css",
        DUE_PILL = "due-pill.css",
        VALUE_DIFF = "value-diff.css",
        SOURCE_CHIP = "source-chip.css",
        FILTER_CHIP = "filter-chip.css",
        STAT_CARD = "stat-card.css",
        CODE_BLOCK = "code-block.css",
        DIFF_VIEWER = "diff-viewer.css",
        MARKDOWN = "markdown.css",
        MARKDOWN_EDITOR = "markdown-editor.css",
        CHIP = "chip.css",
        KBD = "kbd.css",
        AVATAR = "avatar.css",
        CARD = "card.css",
        INPUT = "input.css",
        SEARCH_FIELD = "search-field.css",
        CALLOUT = "callout.css",
        EMPTY_STATE = "empty-state.css",
        PROGRESS_BAR = "progress-bar.css",
        TABS = "tabs.css",
        DATA_TABLE = "data-table.css",
        RESIZABLE = "resizable.css",
        SIDEBAR_LAYOUT = "sidebar-layout.css",
        CHECKBOX = "checkbox.css",
        SWITCH = "switch.css",
        RADIO_GROUP = "radio-group.css",
        COLLAPSIBLE = "collapsible.css",
        ACCORDION = "accordion.css",
        TOOLTIP = "tooltip.css",
        CHART = "chart.css",
        CHART_FLOW = "chart-flow.css",
        CHART_EXTRAS = "chart-extras.css",
        CHART_PLOTS = "chart-plots.css",
        CHART_STATISTICAL = "chart-statistical.css",
        CHART_CARTESIAN = "chart-cartesian.css",
        CHART_HIERARCHY = "chart-hierarchy.css",
        CHART_DISTRIBUTION = "chart-distribution.css",
        CHART_RADIAL = "chart-radial.css",
        CHART_SCIENCE = "chart-science.css",
        CHART_TIME = "chart-time.css",
        CHART_CALENDAR = "chart-calendar.css",
        CHART_GRAPH = "chart-graph.css",
        CHART_TABLE = "chart-table.css",
        CHART_COMPARE = "chart-compare.css",
        CHART_GEO = "chart-geo.css",
        CHART_SETS = "chart-sets.css",
        SELECT = "select.css",
        COMBOBOX = "combobox.css",
        COMMAND_PALETTE = "command-palette.css",
        POPOVER = "popover.css",
        CALENDAR = "calendar.css",
        DATE_PICKER = "date-picker.css",
        COLOR_PICKER = "color-picker.css",
        CRON_BUILDER = "cron-builder.css",
        SIGNATURE_PAD = "signature-pad.css",
        IMAGE_CROPPER = "image-cropper.css",
        DATE_FIELD = "date-field.css",
        DATE_RANGE_PICKER = "date-range-picker.css",
        DURATION_FIELD = "duration-field.css",
        QUANTITY_STEPPER = "quantity-stepper.css",
        TOAST = "toast.css",
        SLIDER = "slider.css",
        RANGE_SLIDER = "range-slider.css",
        RATING = "rating.css",
        TOGGLE_BUTTON = "toggle-button.css",
        FILE_DROPZONE = "file-dropzone.css",
        WIZARD = "wizard.css",
        SEGMENTED = "segmented.css",
        TAGS_INPUT = "tags-input.css",
        ACCESS_EDITOR = "access-editor.css",
        AVAILABILITY_PICKER = "availability-picker.css",
        MEDIA_TIMELINE = "media-timeline.css",
        QUESTIONNAIRE = "questionnaire.css",
        VOICE_INPUT = "voice-input.css",
        VOICE_SESSION = "voice-session.css",
        APP_SHELL = "app-shell.css",
        AUDIO_PLAYER = "audio-player.css",
        AUDIO_WAVEFORM = "audio-waveform.css",
        MASTER_DETAIL = "master-detail.css",
        RECORDING_CONTROL = "recording-control.css",
        TABLE_OF_CONTENTS = "table-of-contents.css",
        TRANSCRIPT_VIEWER = "transcript-viewer.css",
        NUMBER_FIELD = "number-field.css",
        MONEY_FIELD = "money-field.css",
        PHONE_FIELD = "phone-field.css",
        TIME_FIELD = "time-field.css",
        OTP_INPUT = "otp-input.css",
        PASSWORD_FIELD = "password-field.css",
        DIALOG = "dialog.css",
        MENU = "menu.css",
        TREE_VIEW = "tree-view.css",
        KANBAN = "kanban.css",
        DASHBOARD_LAYOUT = "dashboard-layout.css",
        DOCK_LAYOUT = "dock-layout.css",
        GRID = "grid.css",
        CAROUSEL = "carousel.css",
        REVEAL = "reveal.css",
        INLINE_EDIT = "inline-edit.css",
        PRESENCE_BAR = "presence-bar.css",
        LIVE_CURSORS = "live-cursors.css",
        REACTION_BAR = "reaction-bar.css",
        EMOJI_PICKER = "emoji-picker.css",
        TOOL_TRACE = "tool-trace.css",
        MODEL_PICKER = "model-picker.css",
        CITATION_POPOVER = "citation-popover.css",
        ASYNC_VALIDATED_INPUT = "async-validated-input.css",
        SHORTCUT_RECORDER = "shortcut-recorder.css",
        SORTABLE_LIST = "sortable-list.css",
        FILTER_BUILDER = "filter-builder.css",
        TIMELINE = "timeline.css",
        APPROVAL_CARD = "approval-card.css",
        SHEET = "sheet.css",
        HOVER_CARD = "hover-card.css",
        CONTEXT_MENU = "context-menu.css",
        UPLOAD_QUEUE = "upload-queue.css",
        ATTACHMENT = "attachment.css",
        SAVED_VIEWS = "saved-views.css",
        FILE_MANAGER = "file-manager.css",
        GANTT = "gantt.css",
        QR_CODE = "qr-code.css",
        SPREADSHEET_GRID = "spreadsheet-grid.css",
        MENTION_INPUT = "mention-input.css",
        WEEK_SCHEDULER = "week-scheduler.css",
        VIRTUAL_LIST = "virtual-list.css",
        PAGINATION = "pagination.css",
        ALERT_DIALOG = "alert-dialog.css",
        MENUBAR = "menubar.css",
        NAVIGATION_MENU = "navigation-menu.css",
        NOTIFICATION_CENTER = "notification-center.css",
        THEME_MENU = "theme-menu.css",
        BREADCRUMB = "breadcrumb.css",
        LIGHTBOX = "lightbox.css",
        MEDIA_GALLERY = "media-gallery.css",
        SKELETON = "skeleton.css",
        TOOLBAR = "toolbar.css",
        TOUR = "tour.css",
        THEME_EDITOR = "theme-editor.css",
        NODE_EDITOR = "node-editor.css",
        WHITEBOARD = "whiteboard.css",
    }
}

#[cfg(test)]
mod tests {
    use super::{CSS, css};

    #[test]
    fn css_is_the_concatenation_of_every_constant() {
        assert_eq!(css::ALL.concat(), CSS);
    }
}
