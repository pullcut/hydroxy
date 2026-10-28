//! `<VoronoiTreemapChart>`: a tree as nested weighted Voronoi cells inside a circle or a
//! rectangle, each cell's area its share of its parent. Layout comes from
//! `hydroxy_charts::voronoi`.

use hydroxy_charts::{chord, format_value, nav, voronoi};
use leptos::prelude::*;

use crate::plot_kit::colour_or_muted;
use crate::sets_kit::SetsTip;

pub use hydroxy_charts::hierarchy::Tree;
pub use hydroxy_charts::voronoi::Shape;

/// A Voronoi treemap. Cells are coloured by their top-level group, with the group
/// outlines drawn heavier; the larger cells carry their name and value. Hover or touch a
/// cell to read it; focus the chart and use the arrow keys to step through the cells,
/// Home and End to jump, Escape to clear. The same data always draws the same picture.
#[allow(clippy::too_many_lines, reason = "one view! tree; splitting it only adds prop plumbing")]
#[component]
pub fn VoronoiTreemapChart(
    /// The tree. Branch values come from their children.
    #[prop(into)]
    root: Signal<Tree>,
    /// What the chart shows, such as "Cloud spend by service". Used as the accessible name.
    #[prop(into)]
    label: String,
    /// Drawing width in px; the chart scales down to fit its container.
    #[prop(default = 520)]
    width: u16,
    /// Drawing height in px.
    #[prop(default = 520)]
    height: u16,
    /// The outline the cells fill.
    #[prop(optional)]
    shape: Shape,
    /// How many levels to draw, 1 to 4.
    #[prop(default = 2)]
    depth: u8,
    /// Formats values for labels, the tooltip and the table. Default: whole numbers, else one decimal place.
    #[prop(optional, into)]
    format: Option<Callback<f64, String>>,
) -> impl IntoView {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.run(v));
    let (w, h) = (f64::from(width), f64::from(height));
    let layout = Memo::new(move |_| root.with(|r| voronoi::layout(r, w, h, shape, usize::from(depth.clamp(1, 4)))));
    // Only the leaves are marks; branches show through their children.
    let leaves =
        Memo::new(move |_| layout.with(|l| (0..l.cells.len()).filter(|&i| l.cells[i].leaf).collect::<Vec<_>>()));
    let active = RwSignal::new(None::<usize>);
    let pick = move |i: usize| {
        if active.get_untracked() != Some(i) {
            active.set(Some(i));
        }
    };
    let tip = move || active.get().and_then(|i| layout.with(|l| l.tip(i, &fmt)));
    let group_label = format!("{label}. Use the arrow keys to step through the cells.");
    let caption = label;

    view! {
        <figure data-scope="voronoi-treemap-chart" data-part="root" data-active=move || active.get().map(|_| "") style=format!("max-width: {width}px;")>
            <div data-part="plot">
                <svg
                    data-part="marks"
                    data-shape=match shape {
                        Shape::Circle => "circle",
                        Shape::Rect => "rect",
                    }
                    viewBox=format!("0 0 {width} {height}")
                    tabindex="0"
                    role="group"
                    aria-label=group_label
                    on:pointerleave=move |_| active.set(None)
                    on:blur=move |_| active.set(None)
                    on:keydown=move |ev| {
                        let list = leaves.get_untracked();
                        let at = active.get_untracked().and_then(|a| list.iter().position(|&i| i == a));
                        if let Some(next) = nav::list_key(&ev.key(), at, list.len()) {
                            ev.prevent_default();
                            active.set(next.map(|k| list[k]));
                        }
                    }
                >
                    {move || {
                        let l = layout.get();
                        let cells = l
                            .cells
                            .iter()
                            .enumerate()
                            .map(|(i, c)| {
                                let leaf = c.leaf;
                                view! {
                                    <path
                                        data-part="cell"
                                        data-depth=c.depth
                                        data-leaf=leaf.then_some("")
                                        data-active=move || (active.get() == Some(i)).then_some("")
                                        d=c.d.clone()
                                        fill=chord::colour(c.colour, l.groups)
                                        style=format!("--i: {}; --d: {};", c.path.last().copied().unwrap_or(0), c.depth - 1)
                                        on:pointerenter=move |_| {
                                            if leaf {
                                                pick(i);
                                            }
                                        }
                                        on:pointerdown=move |_| {
                                            if leaf {
                                                pick(i);
                                            }
                                        }
                                    ></path>
                                }
                            })
                            .collect_view();
                        let borders = l
                            .cells
                            .iter()
                            .filter(|c| !c.leaf)
                            .map(|c| view! { <path data-part="border" data-depth=c.depth d=c.d.clone()></path> })
                            .collect_view();
                        let labels = l
                            .cells
                            .iter()
                            .filter(|c| c.leaf && c.label_fits())
                            .map(|c| {
                                view! {
                                    <text data-part="label" x=c.centroid.0 y=c.centroid.1 - 6.0 text-anchor="middle">
                                        {c.name.clone()}
                                        <tspan data-part="label-value" x=c.centroid.0 dy="14">{fmt(c.value)}</tspan>
                                    </text>
                                }
                            })
                            .collect_view();
                        view! {
                            <g data-part="cells">{cells}</g>
                            <g data-part="borders">{borders}</g>
                            <path data-part="outline" d=l.outline.clone()></path>
                            <g data-part="labels">{labels}</g>
                        }
                    }}
                </svg>
                {move || tip().map(|tip| view! { <SetsTip tip=tip /> })}
            </div>
            {move || {
                let l = layout.get();
                (l.groups > 1)
                    .then(|| {
                        let mut items: Vec<(String, String)> = l
                            .cells
                            .iter()
                            .filter(|c| c.depth == 1 && chord::slot(c.colour, l.groups).is_some())
                            .map(|c| (chord::colour(c.colour, l.groups), c.name.clone()))
                            .collect();
                        if l.groups > 6 {
                            items.push((colour_or_muted(6, 6), "Other".to_string()));
                        }
                        view! {
                            <ul data-part="legend" aria-hidden="true">
                                {items
                                    .into_iter()
                                    .map(|(c, t)| {
                                        view! {
                                            <li data-part="legend-item">
                                                <span data-part="swatch" style=format!("background: {c};")></span>
                                                {t}
                                            </li>
                                        }
                                    })
                                    .collect_view()}
                            </ul>
                        }
                    })
            }}
            <span data-part="sr-live" aria-live="polite">{move || tip().map(|t| t.announce()).unwrap_or_default()}</span>
            <table data-part="sr-table">
                <caption>{caption}</caption>
                <thead>
                    <tr>
                        <th scope="col">"Item"</th>
                        <th scope="col">"Value"</th>
                    </tr>
                </thead>
                <tbody>
                    {move || {
                        root.with(Tree::flatten)
                            .into_iter()
                            .map(|(name, v)| {
                                view! {
                                    <tr>
                                        <th scope="row">{name}</th>
                                        <td>{fmt(v)}</td>
                                    </tr>
                                }
                            })
                            .collect_view()
                    }}
                </tbody>
            </table>
        </figure>
    }
}
