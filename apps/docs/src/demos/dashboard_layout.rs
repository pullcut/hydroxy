use hydroxy_leptos::dashboard_layout::{DashboardItem, DashboardLayout, Widget};
use hydroxy_leptos::sparkline::Sparkline;
use hydroxy_leptos::stat_card::{DeltaTone, Jewel, StatCard};
use leptos::prelude::*;

fn revenue() -> impl IntoView {
    view! { <StatCard label="This month" value="£48.2k" delta="+8%" delta_tone=DeltaTone::Up jewel=Jewel::Emerald /> }
}

fn signups() -> impl IntoView {
    view! { <StatCard label="This week" value="312" delta="−3%" delta_tone=DeltaTone::Down jewel=Jewel::Sapphire /> }
}

fn churn() -> impl IntoView {
    view! { <StatCard label="Last 30 days" value="1.9" unit="%" caption="steady" jewel=Jewel::Amethyst /> }
}

fn trend() -> impl IntoView {
    let values = Signal::stored(vec![12.0, 14.0, 13.0, 17.0, 16.0, 21.0, 24.0, 23.0, 28.0]);
    view! { <Sparkline values=values label="Weekly active users" width=420 height=80 area=true /> }
}

fn notes() -> impl IntoView {
    view! { <p style="margin: 0;">"Resize a tile from its corner. Shift with the arrow keys resizes from the keyboard."</p> }
}

#[component]
pub fn Demo() -> impl IntoView {
    let (last, set_last) = signal(String::from("Drag a tile by its handle, or focus a handle and press Enter."));
    let widgets = vec![
        Widget::new("revenue", "Revenue", revenue),
        Widget::new("signups", "Signups", signups),
        Widget::new("churn", "Churn", churn),
        Widget::new("trend", "Weekly active", trend),
        Widget::new("notes", "Notes", notes),
    ];
    let items = vec![
        DashboardItem::new("revenue", 0, 0, 4, 3),
        DashboardItem::new("signups", 4, 0, 4, 3),
        DashboardItem::new("churn", 8, 0, 4, 3),
        DashboardItem::new("trend", 0, 3, 8, 3),
        DashboardItem::new("notes", 8, 3, 4, 3),
    ];

    let on_change = move |items: Vec<DashboardItem>| {
        let line: Vec<String> = items.iter().map(|i| format!("{} ({}, {}) {}×{}", i.id, i.x, i.y, i.w, i.h)).collect();
        set_last.set(line.join(" · "));
    };

    view! {
        <div style="display: grid; gap: var(--space-4);">
            <DashboardLayout
                widgets=widgets
                default_items=items
                on_layout_change=on_change
            />
            <p style="margin: 0; font: var(--text-label); color: var(--text-3);">{last}</p>
        </div>
    }
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "dashboard-layout",
    name: "Dashboard layout",
    group: "Layout",
    summary: "Widget tiles on a column grid. Drag a tile by its handle, resize it from the corner, or do both from the keyboard. Tiles in the way make room.",
    demo: || view! { <Demo /> }.into_any(),
    leptos: include_str!("dashboard_layout.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/dashboard_layout.rs"),
    parts: &[
        &["DashboardLayout", "root", "Wrapper for the board and its screen reader text"],
        &[
            "",
            "board",
            "role=group CSS grid. --ox-dashboard-columns, --ox-dashboard-row and --ox-dashboard-gap set by inline style. data-dragging during a pointer drag",
        ],
        &["", "item", "<article> per widget, placed with grid-column and grid-row. data-active while moving"],
        &["", "handle", "<button> in the tile header. aria-pressed while picked up with the keyboard"],
        &["", "resize", "Corner grip for pointers. Hidden from assistive tech"],
        &["", "instructions", "Visually hidden keyboard help, linked from every handle by aria-describedby"],
        &["", "live", "role=status region that announces pick-up, moves, drop and cancel"],
    ],
    keys: &[
        &["Enter / Space", "On a handle: picks the tile up, or drops it"],
        &["Arrow keys", "While carrying: move the tile one cell"],
        &["Shift + Arrow keys", "While carrying: resize the tile one cell"],
        &["Escape", "Puts the tile back where it started"],
        &["Tab", "Moves between handles; leaving a carried tile drops it"],
    ],
    props: &[
        &["widgets", "Vec<Widget>", "Required. Widget::new(id, title, content). Each id needs an item in the layout"],
        &["items", "Option<Vec<DashboardItem>>", "Controlled layout. Pair with on_layout_change"],
        &[
            "default_items",
            "Vec<DashboardItem>",
            "Initial layout when uncontrolled. DashboardItem::new(id, x, y, w, h)",
        ],
        &["columns", "u32", "Columns in the grid. Default 12"],
        &["row_height", "f64", "Row height in px. Default 72"],
        &["gap", "f64", "Gap between tiles in px. Default 12"],
        &["disabled", "bool", "Block every move and resize"],
        &[
            "on_layout_change",
            "Callback<Vec<DashboardItem>>",
            "Called with the whole layout when a gesture ends in a change",
        ],
        &["label", "String", "Accessible name for the board"],
    ],
    events: &[],
};
