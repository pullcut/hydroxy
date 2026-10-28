//! Every hydroxy component in one desktop window. Run with `cargo run -p dioxus-gallery`.
//! Each module is also shown as the Dioxus code on its docs page.

#![allow(missing_docs, clippy::must_use_candidate, reason = "example app")]

use dioxus::prelude::*;

mod accordion;
mod approval_card;
mod kanban;
mod inline_edit;
mod presence_bar;
mod live_cursors;
mod reaction_bar;
mod emoji_picker;
mod tool_trace;
mod model_picker;
mod citation_popover;
mod tour;
mod toolbar;
mod skeleton;
mod media_gallery;
mod lightbox;
mod breadcrumb;
mod async_validated_input;
mod shortcut_recorder;
mod sortable_list;
mod filter_builder;
mod timeline;
mod tree_view;
mod activity;
mod insight_note;
mod composer;
mod thread;
mod thread_list;
mod assistant_composer;
mod assistant_mcp;
mod assistant_modal;
mod assistant_thread;
mod assistant_thread_list;
mod message_scroller;
mod comment_thread;
mod log_stream;
mod diff_viewer;
mod json_inspector;
mod terminal;
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
mod color_picker;
mod cron_builder;
mod theme_editor;
mod qr_code;
mod spreadsheet_grid;
mod mention_input;
mod week_scheduler;
mod virtual_list;
mod signature_pad;
mod image_cropper;
mod date_field;
mod date_range_picker;
mod duration_field;
mod quantity_stepper;
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
mod alert_dialog;
mod menubar;
mod navigation_menu;
mod notification_center;
mod theme_menu;
mod popover;
mod progress_bar;
mod radio_group;
mod range_slider;
mod rating;
mod toggle_button;
mod file_dropzone;
mod upload_queue;
mod attachment;
mod saved_views;
mod file_manager;
mod gantt;
mod wizard;
mod search_field;
mod segmented;
mod select;
mod slider;
mod switch;
mod tabs;
mod data_table;
mod resizable;
mod dashboard_layout;
mod dock_layout;
mod grid;
mod carousel;
mod reveal;
mod sidebar_layout;
mod bar_chart;
mod chart_frame;
mod heatmap_chart;
mod radial_charts;
mod diverging_bar_chart;
mod span_chart;
mod pictogram_chart;
mod scatter_chart;
mod bump_chart;
mod contour_chart;
mod geo_map;
mod bubble_map_chart;
mod hexbin_map_chart;
mod radar_chart;
mod slope_chart;
mod smith_chart;
mod donut_chart;
mod sankey_chart;
mod network_chart;
mod arc_diagram_chart;
mod hive_plot_chart;
mod edge_bundling_chart;
mod cartogram_chart;
mod flow_map_chart;
mod raincloud_chart;
mod forest_plot_chart;
mod chord_chart;
mod upset_chart;
mod venn_chart;
mod voronoi_treemap_chart;
mod nested_donut_chart;
mod alluvial_chart;
mod adjacency_matrix_chart;
mod dependency_graph_chart;
mod treemap_chart;
mod sunburst_chart;
mod bubble_pack_chart;
mod icicle_chart;
mod flame_graph_chart;
mod tree_chart;
mod line_chart;
mod histogram;
mod box_plot;
mod candlestick_chart;
mod composed_chart;
mod strip_plot_chart;
mod beeswarm_chart;
mod density_chart;
mod ridgeline_chart;
mod violin_chart;
mod ohlc_chart;
mod depth_chart;
mod renko_chart;
mod timeline_chart;
mod sparkline;
mod stacked_bar_chart;
mod small_multiples;
mod chart_table;
mod cohort_chart;
mod parallel_coordinates_chart;
mod pie_chart;
mod polar_area_chart;
mod funnel_chart;
mod bullet_chart;
mod waffle_chart;
mod marimekko_chart;
mod metric_card;
mod stat_ring;
mod word_cloud_chart;
mod ternary_chart;
mod dot_plot_chart;
mod error_bar_chart;
mod pareto_chart;
mod calendar_heatmap_chart;
mod clock_chart;
mod cycle_plot_chart;
mod horizon_chart;
mod spiral_chart;
mod waterfall_chart;
mod lollipop_chart;
mod grouped_bar_chart;
mod stacked_area_chart;
mod threshold_chart;
mod split_line_chart;
mod confidence_band_chart;
mod forecast_fan_chart;
mod qq_chart;
mod bland_altman_chart;
mod survival_chart;
mod ecdf_chart;
mod quiver_chart;
mod spectrogram_chart;
mod marey_chart;
mod step_chart;
mod connected_scatter_chart;
mod range_area_chart;
mod bar_race_chart;
mod tag;
mod tags_input;
mod access_editor;
mod availability_picker;
mod media_timeline;
mod questionnaire;
mod voice_input;
mod transcript_viewer;
mod table_of_contents;
mod recording_control;
mod master_detail;
mod audio_waveform;
mod audio_player;
mod app_shell;
mod voice_session;
mod number_field;
mod money_field;
mod phone_field;
mod time_field;
mod otp_input;
mod password_field;
mod toast;
mod tooltip;
mod node_editor;
mod whiteboard;

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        style { {hydroxy_tokens::GOOGLE_FONTS} {hydroxy_tokens::CSS} }
        main { style: "max-width: 720px; margin: 0 auto; padding: 32px; display: grid; gap: 48px;",
            section { h2 { "Tabs" } tabs::TabsDemo {} }
            section { h2 { "Data table" } data_table::DataTableDemo {} }
            section { h2 { "Sidebar layout" } sidebar_layout::SidebarLayoutDemo {} }
            section { h2 { "Resizable panels" } resizable::ResizableDemo {} }
            section { h2 { "Dashboard layout" } dashboard_layout::DashboardLayoutDemo {} }
            section { h2 { "Dock layout" } dock_layout::DockLayoutDemo {} }
            section { h2 { "Grid" } grid::GridDemo {} }
            section { h2 { "Carousel" } carousel::CarouselDemo {} }
            section { h2 { "Reveal" } reveal::RevealDemo {} }
            section { h2 { "Activity feed" } activity::ActivityFeedDemo {} }
            section { h2 { "Insight note" } insight_note::InsightNoteDemo {} }
            section { h2 { "Composer" } composer::ComposerDemo {} }
            section { h2 { "Thread" } thread::ThreadDemo {} }
            section { h2 { "Thread list" } thread_list::ThreadListDemo {} }
            section { h2 { "Assistant composer" } assistant_composer::AssistantComposerDemo {} }
            section { h2 { "Assistant MCP" } assistant_mcp::AssistantMcpDemo {} }
            section { h2 { "Assistant thread" } assistant_thread::AssistantThreadDemo {} }
            section { h2 { "Assistant thread list" } assistant_thread_list::AssistantThreadListDemo {} }
            section { h2 { "Assistant modal" } assistant_modal::AssistantModalDemo {} }
            section { h2 { "Message scroller" } message_scroller::MessageScrollerDemo {} }
            section { h2 { "Comment thread" } comment_thread::CommentThreadDemo {} }
            section { h2 { "Log stream" } log_stream::LogStreamDemo {} }
            section { h2 { "Diff viewer" } diff_viewer::DiffViewerDemo {} }
            section { h2 { "JSON inspector" } json_inspector::JsonInspectorDemo {} }
            section { h2 { "Terminal" } terminal::TerminalDemo {} }
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
            section { h2 { "Diverging bar chart" } diverging_bar_chart::DivergingBarChartDemo {} }
            section { h2 { "Span chart" } span_chart::SpanChartDemo {} }
            section { h2 { "Pictogram chart" } pictogram_chart::PictogramChartDemo {} }
            section { h2 { "Bump chart" } bump_chart::BumpChartDemo {} }
            section { h2 { "Slope chart" } slope_chart::SlopeChartDemo {} }
            section { h2 { "Radar chart" } radar_chart::RadarChartDemo {} }
            section { h2 { "Contour chart" } contour_chart::ContourChartDemo {} }
            section { h2 { "Geo map" } geo_map::GeoMapDemo {} }
            section { h2 { "Bubble map" } bubble_map_chart::BubbleMapChartDemo {} }
            section { h2 { "Hexbin map" } hexbin_map_chart::HexbinMapChartDemo {} }
            section { h2 { "Smith chart" } smith_chart::SmithChartDemo {} }
            section { h2 { "Donut chart" } donut_chart::DonutChartDemo {} }
            section { h2 { "Sankey chart" } sankey_chart::SankeyChartDemo {} }
            section { h2 { "Network chart" } network_chart::NetworkChartDemo {} }
            section { h2 { "Arc diagram" } arc_diagram_chart::ArcDiagramChartDemo {} }
            section { h2 { "Hive plot" } hive_plot_chart::HivePlotChartDemo {} }
            section { h2 { "Edge bundling" } edge_bundling_chart::EdgeBundlingChartDemo {} }
            section { h2 { "Cartogram" } cartogram_chart::CartogramChartDemo {} }
            section { h2 { "Flow map" } flow_map_chart::FlowMapChartDemo {} }
            section { h2 { "Raincloud plot" } raincloud_chart::RaincloudChartDemo {} }
            section { h2 { "Forest plot" } forest_plot_chart::ForestPlotChartDemo {} }
            section { h2 { "Chord chart" } chord_chart::ChordChartDemo {} }
            section { h2 { "UpSet chart" } upset_chart::UpsetChartDemo {} }
            section { h2 { "Venn chart" } venn_chart::VennChartDemo {} }
            section { h2 { "Voronoi treemap" } voronoi_treemap_chart::VoronoiTreemapChartDemo {} }
            section { h2 { "Nested donut chart" } nested_donut_chart::NestedDonutChartDemo {} }
            section { h2 { "Alluvial chart" } alluvial_chart::AlluvialChartDemo {} }
            section { h2 { "Adjacency matrix chart" } adjacency_matrix_chart::AdjacencyMatrixChartDemo {} }
            section { h2 { "Dependency graph chart" } dependency_graph_chart::DependencyGraphChartDemo {} }
            section { h2 { "Treemap chart" } treemap_chart::TreemapChartDemo {} }
            section { h2 { "Sunburst chart" } sunburst_chart::SunburstChartDemo {} }
            section { h2 { "Bubble pack chart" } bubble_pack_chart::BubblePackChartDemo {} }
            section { h2 { "Icicle chart" } icicle_chart::IcicleChartDemo {} }
            section { h2 { "Flame graph chart" } flame_graph_chart::FlameGraphChartDemo {} }
            section { h2 { "Tree chart" } tree_chart::TreeChartDemo {} }
            section { h2 { "Line chart" } line_chart::LineChartDemo {} }
            section { h2 { "Histogram" } histogram::HistogramDemo {} }
            section { h2 { "Box plot" } box_plot::BoxPlotDemo {} }
            section { h2 { "Candlestick chart" } candlestick_chart::CandlestickChartDemo {} }
            section { h2 { "Composed chart" } composed_chart::ComposedChartDemo {} }
            section { h2 { "Strip plot" } strip_plot_chart::StripPlotChartDemo {} }
            section { h2 { "Beeswarm chart" } beeswarm_chart::BeeswarmChartDemo {} }
            section { h2 { "Density chart" } density_chart::DensityChartDemo {} }
            section { h2 { "Ridgeline chart" } ridgeline_chart::RidgelineChartDemo {} }
            section { h2 { "Violin chart" } violin_chart::ViolinChartDemo {} }
            section { h2 { "OHLC chart" } ohlc_chart::OhlcChartDemo {} }
            section { h2 { "Depth chart" } depth_chart::DepthChartDemo {} }
            section { h2 { "Renko chart" } renko_chart::RenkoChartDemo {} }
            section { h2 { "Timeline chart" } timeline_chart::TimelineChartDemo {} }
            section { h2 { "Stacked bar chart" } stacked_bar_chart::StackedBarChartDemo {} }
            section { h2 { "Small multiples" } small_multiples::SmallMultiplesDemo {} }
            section { h2 { "Chart table" } chart_table::ChartTableDemo {} }
            section { h2 { "Cohort chart" } cohort_chart::CohortChartDemo {} }
            section { h2 { "Parallel coordinates" } parallel_coordinates_chart::ParallelCoordinatesChartDemo {} }
            section { h2 { "Pie chart" } pie_chart::PieChartDemo {} }
            section { h2 { "Polar area chart" } polar_area_chart::PolarAreaChartDemo {} }
            section { h2 { "Funnel chart" } funnel_chart::FunnelChartDemo {} }
            section { h2 { "Bullet chart" } bullet_chart::BulletChartDemo {} }
            section { h2 { "Waffle chart" } waffle_chart::WaffleChartDemo {} }
            section { h2 { "Marimekko chart" } marimekko_chart::MarimekkoChartDemo {} }
            section { h2 { "Metric card" } metric_card::MetricCardDemo {} }
            section { h2 { "Stat ring" } stat_ring::StatRingDemo {} }
            section { h2 { "Word cloud" } word_cloud_chart::WordCloudChartDemo {} }
            section { h2 { "Ternary chart" } ternary_chart::TernaryChartDemo {} }
            section { h2 { "Dot plot chart" } dot_plot_chart::DotPlotChartDemo {} }
            section { h2 { "Error bar chart" } error_bar_chart::ErrorBarChartDemo {} }
            section { h2 { "Pareto chart" } pareto_chart::ParetoChartDemo {} }
            section { h2 { "Calendar heatmap chart" } calendar_heatmap_chart::CalendarHeatmapChartDemo {} }
            section { h2 { "Clock chart" } clock_chart::ClockChartDemo {} }
            section { h2 { "Cycle plot chart" } cycle_plot_chart::CyclePlotChartDemo {} }
            section { h2 { "Horizon chart" } horizon_chart::HorizonChartDemo {} }
            section { h2 { "Spiral chart" } spiral_chart::SpiralChartDemo {} }
            section { h2 { "Waterfall chart" } waterfall_chart::WaterfallChartDemo {} }
            section { h2 { "Lollipop chart" } lollipop_chart::LollipopChartDemo {} }
            section { h2 { "Grouped bar chart" } grouped_bar_chart::GroupedBarChartDemo {} }
            section { h2 { "Stacked area chart" } stacked_area_chart::StackedAreaChartDemo {} }
            section { h2 { "Threshold chart" } threshold_chart::ThresholdChartDemo {} }
            section { h2 { "Split line chart" } split_line_chart::SplitLineChartDemo {} }
            section { h2 { "Confidence band chart" } confidence_band_chart::ConfidenceBandChartDemo {} }
            section { h2 { "Forecast fan chart" } forecast_fan_chart::ForecastFanChartDemo {} }
            section { h2 { "Q-Q chart" } qq_chart::QQChartDemo {} }
            section { h2 { "Bland-Altman chart" } bland_altman_chart::BlandAltmanChartDemo {} }
            section { h2 { "Survival chart" } survival_chart::SurvivalChartDemo {} }
            section { h2 { "ECDF chart" } ecdf_chart::EcdfChartDemo {} }
            section { h2 { "Quiver chart" } quiver_chart::QuiverChartDemo {} }
            section { h2 { "Spectrogram chart" } spectrogram_chart::SpectrogramChartDemo {} }
            section { h2 { "Marey chart" } marey_chart::MareyChartDemo {} }
            section { h2 { "Step chart" } step_chart::StepChartDemo {} }
            section { h2 { "Connected scatter chart" } connected_scatter_chart::ConnectedScatterChartDemo {} }
            section { h2 { "Range area chart" } range_area_chart::RangeAreaChartDemo {} }
            section { h2 { "Bar race chart" } bar_race_chart::BarRaceChartDemo {} }
            section { h2 { "Sparkline" } sparkline::SparklineDemo {} }
            section { h2 { "Select" } select::SelectDemo {} }
            section { h2 { "Combobox" } combobox::ComboboxDemo {} }
            section { h2 { "Calendar" } calendar::CalendarDemo {} }
            section { h2 { "Date picker" } date_picker::DatePickerDemo {} }
            section { h2 { "Date range picker" } date_range_picker::DateRangePickerDemo {} }
            section { h2 { "Date field" } date_field::DateFieldDemo {} }
            section { h2 { "Duration field" } duration_field::DurationFieldDemo {} }
            section { h2 { "Quantity stepper" } quantity_stepper::QuantityStepperDemo {} }
            section { h2 { "Color picker" } color_picker::ColorPickerDemo {} }
            section { h2 { "Cron builder" } cron_builder::CronBuilderDemo {} }
            section { h2 { "Theme editor" } theme_editor::ThemeEditorDemo {} }
            section { h2 { "QR code" } qr_code::QrCodeDemo {} }
            section { h2 { "Spreadsheet grid" } spreadsheet_grid::SpreadsheetGridDemo {} }
            section { h2 { "Mention input" } mention_input::MentionInputDemo {} }
            section { h2 { "Week scheduler" } week_scheduler::WeekSchedulerDemo {} }
            section { h2 { "Virtual list" } virtual_list::VirtualListDemo {} }
            section { h2 { "Signature pad" } signature_pad::SignaturePadDemo {} }
            section { h2 { "Image cropper" } image_cropper::ImageCropperDemo {} }
            section { h2 { "Command palette" } command_palette::CommandPaletteDemo {} }
            section { h2 { "Popover" } popover::PopoverDemo {} }
            section { h2 { "Toast" } toast::ToastDemo {} }
            section { h2 { "Slider" } slider::SliderDemo {} }
            section { h2 { "Range slider" } range_slider::RangeSliderDemo {} }
            section { h2 { "Rating" } rating::RatingDemo {} }
            section { h2 { "Toggle button" } toggle_button::ToggleButtonDemo {} }
            section { h2 { "File dropzone" } file_dropzone::FileDropzoneDemo {} }
            section { h2 { "Upload queue" } upload_queue::UploadQueueDemo {} }
            section { h2 { "Attachment" } attachment::AttachmentDemo {} }
            section { h2 { "Saved views" } saved_views::SavedViewsDemo {} }
            section { h2 { "File manager" } file_manager::FileManagerDemo {} }
            section { h2 { "Gantt chart" } gantt::GanttDemo {} }
            section { h2 { "Wizard" } wizard::WizardDemo {} }
            section { h2 { "Segmented control" } segmented::SegmentedDemo {} }
            section { h2 { "Tags input" } tags_input::TagsInputDemo {} }
            section { h2 { "Questionnaire" } questionnaire::QuestionnaireDemo {} }
            section { h2 { "Access editor" } access_editor::AccessEditorDemo {} }
            section { h2 { "Availability picker" } availability_picker::AvailabilityPickerDemo {} }
            section { h2 { "Media timeline" } media_timeline::MediaTimelineDemo {} }
            section { h2 { "Voice input" } voice_input::VoiceInputDemo {} }
            section { h2 { "Transcript viewer" } transcript_viewer::TranscriptViewerDemo {} }
            section { h2 { "Table of contents" } table_of_contents::TableOfContentsDemo {} }
            section { h2 { "Recording control" } recording_control::RecordingControlDemo {} }
            section { h2 { "Master detail" } master_detail::MasterDetailDemo {} }
            section { h2 { "Audio waveform" } audio_waveform::AudioWaveformDemo {} }
            section { h2 { "Audio player" } audio_player::AudioPlayerDemo {} }
            section { h2 { "App shell" } app_shell::AppShellDemo {} }
            section { h2 { "Voice session" } voice_session::VoiceSessionDemo {} }
            section { h2 { "Number field" } number_field::NumberFieldDemo {} }
            section { h2 { "Money field" } money_field::MoneyFieldDemo {} }
            section { h2 { "Phone field" } phone_field::PhoneFieldDemo {} }
            section { h2 { "Time field" } time_field::TimeFieldDemo {} }
            section { h2 { "OTP input" } otp_input::OtpInputDemo {} }
            section { h2 { "Password field" } password_field::PasswordFieldDemo {} }
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
            section { h2 { "Alert dialog" } alert_dialog::AlertDialogDemo {} }
            section { h2 { "Menubar" } menubar::MenubarDemo {} }
            section { h2 { "Navigation menu" } navigation_menu::NavigationMenuDemo {} }
            section { h2 { "Notification center" } notification_center::NotificationCenterDemo {} }
            section { h2 { "Theme menu" } theme_menu::ThemeMenuDemo {} }
            section { h2 { "Checkbox" } checkbox::CheckboxDemo {} }
            section { h2 { "Switch" } switch::SwitchDemo {} }
            section { h2 { "RadioGroup" } radio_group::RadioGroupDemo {} }
            section { h2 { "Collapsible" } collapsible::CollapsibleDemo {} }
            section { h2 { "Accordion" } accordion::AccordionDemo {} }
            section { h2 { "TreeView" } tree_view::TreeViewDemo {} }
            section { h2 { "KanbanBoard" } kanban::KanbanDemo {} }
            section { h2 { "InlineEdit" } inline_edit::InlineEditDemo {} }
            section { h2 { "PresenceBar" } presence_bar::PresenceBarDemo {} }
            section { h2 { "LiveCursors" } live_cursors::LiveCursorsDemo {} }
            section { h2 { "ReactionBar" } reaction_bar::ReactionBarDemo {} }
            section { h2 { "EmojiPicker" } emoji_picker::EmojiPickerDemo {} }
            section { h2 { "ToolTrace" } tool_trace::ToolTraceDemo {} }
            section { h2 { "ModelPicker" } model_picker::ModelPickerDemo {} }
            section { h2 { "CitationPopover" } citation_popover::CitationPopoverDemo {} }
            section { h2 { "Breadcrumb" } breadcrumb::BreadcrumbDemo {} }
            section { h2 { "Toolbar" } toolbar::ToolbarDemo {} }
            section { h2 { "Skeleton" } skeleton::SkeletonDemo {} }
            section { h2 { "Tour" } tour::TourDemo {} }
            section { h2 { "Lightbox" } lightbox::LightboxDemo {} }
            section { h2 { "MediaGallery" } media_gallery::MediaGalleryDemo {} }
            section { h2 { "AsyncValidatedInput" } async_validated_input::AsyncValidatedInputDemo {} }
            section { h2 { "ShortcutRecorder" } shortcut_recorder::ShortcutRecorderDemo {} }
            section { h2 { "SortableList" } sortable_list::SortableListDemo {} }
            section { h2 { "FilterBuilder" } filter_builder::FilterBuilderDemo {} }
            section { h2 { "Timeline" } timeline::TimelineDemo {} }
            section { h2 { "ApprovalCard" } approval_card::ApprovalCardDemo {} }
            section { h2 { "Node editor" } node_editor::NodeEditorDemo {} }
            section { h2 { "Whiteboard" } whiteboard::WhiteboardDemo {} }
        }
    }
}
