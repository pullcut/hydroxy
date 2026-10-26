//! Every oxide component in one desktop window. Run with `cargo run -p dioxus-gallery`.
//! Each module is also shown as the Dioxus code on its docs page.

#![allow(missing_docs, clippy::must_use_candidate, reason = "example app")]

use dioxus::prelude::*;

mod accordion;
mod approval_card;
mod kanban;
mod timeline;
mod tree_view;
mod activity;
mod insight_note;
mod composer;
mod chat_bubble;
mod nav_section;
mod nav_item;
mod step_track;
mod check_list;
mod doc_card;
mod space_icon;
mod code_block;
mod stat_card;
mod filter_chip;
mod source_chip;
mod value_diff;
mod due_pill;
mod priority_mark;
mod status_diamond;
mod avatar;
mod badge;
mod button;
mod callout;
mod card;
mod calendar;
mod checkbox;
mod date_picker;
mod chip;
mod collapsible;
mod combobox;
mod command_palette;
mod dialog;
mod empty_state;
mod icon_button;
mod input;
mod kbd;
mod markdown;
mod markdown_editor;
mod menu;
mod sheet;
mod hover_card;
mod context_menu;
mod pagination;
mod popover;
mod progress_bar;
mod radio_group;
mod range_slider;
mod rating;
mod toggle_button;
mod file_dropzone;
mod wizard;
mod search_field;
mod segmented;
mod select;
mod slider;
mod switch;
mod tabs;
mod data_table;
mod resizable;
mod sidebar_layout;
mod bar_chart;
mod chart_frame;
mod heatmap_chart;
mod radial_charts;
mod scatter_chart;
mod bump_chart;
mod contour_chart;
mod geo_map;
mod radar_chart;
mod slope_chart;
mod smith_chart;
mod donut_chart;
mod sankey_chart;
mod network_chart;
mod treemap_chart;
mod sunburst_chart;
mod line_chart;
mod histogram;
mod box_plot;
mod candlestick_chart;
mod timeline_chart;
mod sparkline;
mod stacked_bar_chart;
mod tag;
mod tags_input;
mod number_field;
mod time_field;
mod otp_input;
mod toast;
mod tooltip;

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        style { {oxide_tokens::GOOGLE_FONTS} {oxide_tokens::CSS} }
        main { style: "max-width: 720px; margin: 0 auto; padding: 32px; display: grid; gap: 48px;",
            section { h2 { "Tabs" } tabs::TabsDemo {} }
            section { h2 { "Data table" } data_table::DataTableDemo {} }
            section { h2 { "Sidebar layout" } sidebar_layout::SidebarLayoutDemo {} }
            section { h2 { "Resizable panels" } resizable::ResizableDemo {} }
            section { h2 { "Activity feed" } activity::ActivityFeedDemo {} }
            section { h2 { "Insight note" } insight_note::InsightNoteDemo {} }
            section { h2 { "Composer" } composer::ComposerDemo {} }
            section { h2 { "Chat bubble" } chat_bubble::ChatBubbleDemo {} }
            section { h2 { "Nav section" } nav_section::NavSectionDemo {} }
            section { h2 { "Nav item" } nav_item::NavItemDemo {} }
            section { h2 { "Step track" } step_track::StepTrackDemo {} }
            section { h2 { "Check list" } check_list::CheckListDemo {} }
            section { h2 { "Doc card" } doc_card::DocCardDemo {} }
            section { h2 { "Space icon" } space_icon::SpaceIconDemo {} }
            section { h2 { "Code block" } code_block::CodeBlockDemo {} }
            section { h2 { "Stat card" } stat_card::StatCardDemo {} }
            section { h2 { "Filter chip" } filter_chip::FilterChipDemo {} }
            section { h2 { "Source chip" } source_chip::SourceChipDemo {} }
            section { h2 { "Value diff" } value_diff::ValueDiffDemo {} }
            section { h2 { "Due pill" } due_pill::DuePillDemo {} }
            section { h2 { "Priority mark" } priority_mark::PriorityMarkDemo {} }
            section { h2 { "Status diamond" } status_diamond::StatusDiamondDemo {} }
            section { h2 { "Bar chart" } bar_chart::BarChartDemo {} }
            section { h2 { "Chart frame" } chart_frame::ChartFrameDemo {} }
            section { h2 { "Scatter chart" } scatter_chart::ScatterChartDemo {} }
            section { h2 { "Heatmap chart" } heatmap_chart::HeatmapChartDemo {} }
            section { h2 { "Radial charts" } radial_charts::RadialChartsDemo {} }
            section { h2 { "Bump chart" } bump_chart::BumpChartDemo {} }
            section { h2 { "Slope chart" } slope_chart::SlopeChartDemo {} }
            section { h2 { "Radar chart" } radar_chart::RadarChartDemo {} }
            section { h2 { "Contour chart" } contour_chart::ContourChartDemo {} }
            section { h2 { "Geo map" } geo_map::GeoMapDemo {} }
            section { h2 { "Smith chart" } smith_chart::SmithChartDemo {} }
            section { h2 { "Donut chart" } donut_chart::DonutChartDemo {} }
            section { h2 { "Sankey chart" } sankey_chart::SankeyChartDemo {} }
            section { h2 { "Network chart" } network_chart::NetworkChartDemo {} }
            section { h2 { "Treemap chart" } treemap_chart::TreemapChartDemo {} }
            section { h2 { "Sunburst chart" } sunburst_chart::SunburstChartDemo {} }
            section { h2 { "Line chart" } line_chart::LineChartDemo {} }
            section { h2 { "Histogram" } histogram::HistogramDemo {} }
            section { h2 { "Box plot" } box_plot::BoxPlotDemo {} }
            section { h2 { "Candlestick chart" } candlestick_chart::CandlestickChartDemo {} }
            section { h2 { "Timeline chart" } timeline_chart::TimelineChartDemo {} }
            section { h2 { "Stacked bar chart" } stacked_bar_chart::StackedBarChartDemo {} }
            section { h2 { "Sparkline" } sparkline::SparklineDemo {} }
            section { h2 { "Select" } select::SelectDemo {} }
            section { h2 { "Combobox" } combobox::ComboboxDemo {} }
            section { h2 { "Calendar" } calendar::CalendarDemo {} }
            section { h2 { "Date picker" } date_picker::DatePickerDemo {} }
            section { h2 { "Command palette" } command_palette::CommandPaletteDemo {} }
            section { h2 { "Popover" } popover::PopoverDemo {} }
            section { h2 { "Toast" } toast::ToastDemo {} }
            section { h2 { "Slider" } slider::SliderDemo {} }
            section { h2 { "Range slider" } range_slider::RangeSliderDemo {} }
            section { h2 { "Rating" } rating::RatingDemo {} }
            section { h2 { "Toggle button" } toggle_button::ToggleButtonDemo {} }
            section { h2 { "File dropzone" } file_dropzone::FileDropzoneDemo {} }
            section { h2 { "Wizard" } wizard::WizardDemo {} }
            section { h2 { "Segmented control" } segmented::SegmentedDemo {} }
            section { h2 { "Tags input" } tags_input::TagsInputDemo {} }
            section { h2 { "Number field" } number_field::NumberFieldDemo {} }
            section { h2 { "Time field" } time_field::TimeFieldDemo {} }
            section { h2 { "OTP input" } otp_input::OtpInputDemo {} }
            section { h2 { "Button" } button::ButtonDemo {} }
            section { h2 { "IconButton" } icon_button::IconButtonDemo {} }
            section { h2 { "Badge" } badge::BadgeDemo {} }
            section { h2 { "Tag" } tag::TagDemo {} }
            section { h2 { "Chip" } chip::ChipDemo {} }
            section { h2 { "Kbd" } kbd::KbdDemo {} }
            section { h2 { "Markdown" } markdown::MarkdownDemo {} }
            section { h2 { "Markdown editor" } markdown_editor::MarkdownEditorDemo {} }
            section { h2 { "Avatar" } avatar::AvatarDemo {} }
            section { h2 { "Card" } card::CardDemo {} }
            section { h2 { "Input" } input::InputDemo {} }
            section { h2 { "SearchField" } search_field::SearchFieldDemo {} }
            section { h2 { "Callout" } callout::CalloutDemo {} }
            section { h2 { "EmptyState" } empty_state::EmptyStateDemo {} }
            section { h2 { "ProgressBar" } progress_bar::ProgressBarDemo {} }
            section { h2 { "Dialog" } dialog::DialogDemo {} }
            section { h2 { "Menu" } menu::MenuDemo {} }
            section { h2 { "Tooltip" } tooltip::TooltipDemo {} }
            section { h2 { "Sheet" } sheet::SheetDemo {} }
            section { h2 { "Hover card" } hover_card::HoverCardDemo {} }
            section { h2 { "Context menu" } context_menu::ContextMenuDemo {} }
            section { h2 { "Pagination" } pagination::PaginationDemo {} }
            section { h2 { "Checkbox" } checkbox::CheckboxDemo {} }
            section { h2 { "Switch" } switch::SwitchDemo {} }
            section { h2 { "RadioGroup" } radio_group::RadioGroupDemo {} }
            section { h2 { "Collapsible" } collapsible::CollapsibleDemo {} }
            section { h2 { "Accordion" } accordion::AccordionDemo {} }
            section { h2 { "TreeView" } tree_view::TreeViewDemo {} }
            section { h2 { "KanbanBoard" } kanban::KanbanDemo {} }
            section { h2 { "Timeline" } timeline::TimelineDemo {} }
            section { h2 { "ApprovalCard" } approval_card::ApprovalCardDemo {} }
        }
    }
}
