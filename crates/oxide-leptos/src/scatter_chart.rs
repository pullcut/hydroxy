//! `<ScatterChart>`: numeric x and y, with optional bubble size and one colour per group.
//! Geometry comes from `oxide_charts::scatter`; dots are positioned with CSS percentages.

use leptos::prelude::*;
use oxide_charts::annotate;
use oxide_charts::zoom::{Axes, Gesture, View};
use oxide_charts::{format_value, raster, scatter};

use crate::chart_frame::provide_export;
use crate::raster_canvas::RasterCanvas;
use crate::line_chart::{gutter, annotation_views, gesture_down, gesture_move, plot_fractions, wheel_zoom};
pub use oxide_charts::annotate::Annotation;
pub use oxide_charts::scale::ScaleKind;
pub use oxide_charts::scatter::{Group, Point};

const MIN_SPAN: f64 = 0.02;

fn colour(i: usize) -> String {
    format!("var(--chart-{})", i % 6 + 1)
}

fn pct(f: f64) -> String {
    format!("{:.3}%", f * 100.0)
}

/// Dots on two numeric axes. Past a few thousand visible dots they draw on a canvas, so
/// hundreds of thousands of points stay smooth to hover and zoom. Hover or touch to read the nearest dot; focus the plot and
/// use the arrow keys to step through dots from left to right, Home and End to jump,
/// Escape to clear.
#[allow(clippy::too_many_lines, reason = "one view! tree; splitting it only adds prop plumbing")]
#[allow(clippy::needless_pass_by_value, reason = "component props are owned")]
#[component]
pub fn ScatterChart(
    /// The groups, one colour each in categorical order. Up to six read well.
    /// Give points a size to draw bubbles.
    #[prop(into)]
    groups: Signal<Vec<Group>>,
    /// What the chart shows, such as "Interviews against satisfaction". Used as the accessible name.
    #[prop(into)]
    label: String,
    /// Caption for the x axis.
    #[prop(optional, into)]
    x_label: Option<String>,
    /// Caption for the y axis.
    #[prop(optional, into)]
    y_label: Option<String>,
    /// Name of the bubble size measure, shown in the tooltip and table.
    #[prop(default = "Size".into(), into)]
    size_label: String,
    /// Plot height in px, excluding the axis labels and legend.
    #[prop(default = 280)]
    height: u16,
    /// Formats x values for the axis, tooltip and table.
    #[prop(optional, into)]
    format_x: Option<Callback<f64, String>>,
    /// Formats y values for the axis, tooltip and table.
    #[prop(optional, into)]
    format_y: Option<Callback<f64, String>>,
    /// Reference lines, bands and labelled points, in data units on both axes.
    /// The y axis grows to fit them.
    #[prop(optional, into)]
    annotations: Signal<Vec<Annotation>>,
    /// Linear (default), Log or Symlog x axis.
    #[prop(optional)]
    x_scale: ScaleKind,
    /// Linear (default), Log or Symlog y axis.
    #[prop(optional)]
    y_scale: ScaleKind,
    /// Wheel or pinch to zoom around the pointer, drag to pan, and `+`/`-`/Shift+arrows on
    /// the focused plot.
    #[prop(optional)]
    zoom: bool,
) -> impl IntoView {
    let fx = move |v: f64| format_x.map_or_else(|| format_value(v), |f| f.run(v));
    let fy = move |v: f64| format_y.map_or_else(|| format_value(v), |f| f.run(v));
    let view = RwSignal::new(View::FULL);
    // Once zoomed, dots stop replaying their entrance every time the view moves.
    let settled = RwSignal::new(false);
    let set_view = move |v: View| {
        settled.set(true);
        view.set(v);
    };
    let gesture = StoredValue::new(Gesture::default());
    let layout = Memo::new(move |_| {
        let extent = annotations.with(|a| annotate::y_extent(a));
        groups.with(|g| scatter::layout_scaled(g, 6, (x_scale, y_scale), view.get(), extent))
    });
    let placed = Memo::new(move |_| {
        let (sx, sy, v) = layout.with(|l| (l.x_scale, l.y_scale, l.view));
        annotations.with(|a| annotate::place(a, |x| v.project_x(sx.map(x)), |y| v.project_y(sy.map(y))))
    });
    // Big clouds draw on a canvas and find the hovered dot through a grid.
    let big = Memo::new(move |_| layout.with(|l| l.dots.len() > raster::DOM_LIMIT));
    let index = Memo::new(move |_| layout.with(|l| (l.dots.len() > raster::DOM_LIMIT).then(|| scatter::Index::new(&l.dots))));
    // Only read on a key press, so zooming never pays for the sort.
    let order = Memo::new(move |_| layout.with(|l| scatter::order(&l.dots)));
    let active = RwSignal::new(None::<usize>);
    let point_of = move |dot: scatter::Dot| groups.with(|g| g.get(dot.group).and_then(|g| g.points.get(dot.index).copied()));
    let size_label_text = size_label.clone();
    let describe = move |i: usize| {
        let dot = layout.with(|l| l.dots.get(i).copied())?;
        let point = point_of(dot)?;
        let name = groups.with(|g| g[dot.group].name.clone());
        let mut text = format!("{name}: {} by {}", fx(point.x), fy(point.y));
        if let Some(s) = point.size {
            text = format!("{text}, {size_label_text} {}", format_value(s));
        }
        Some(text)
    };
    let move_to = move |ev: leptos::ev::PointerEvent| {
        if zoom && gesture.with_value(Gesture::active) {
            if let Some(next) = gesture_move(gesture, &ev, view.get_untracked(), Axes::XY, MIN_SPAN) {
                active.set(None);
                set_view(next);
            }
            return;
        }
        let Some((x, y)) = plot_fractions(&ev) else { return };
        let near = layout.with(|l| {
            index.with(|ix| match ix {
                Some(ix) => ix.nearest(&l.dots, x, y, 0.08),
                None => scatter::nearest(&l.dots, x, y, 0.08),
            })
        });
        if active.get_untracked() != near {
            active.set(near);
        }
    };
    let has_sizes = move || groups.with(|g| g.iter().flat_map(|g| &g.points).any(|p| p.size.is_some()));
    let size_label_table = size_label.clone();
    let x_label_title = x_label.clone();
    let x_head = x_label.clone().unwrap_or_else(|| "X".into());
    let y_head = y_label.clone().unwrap_or_else(|| "Y".into());
    let size_head = size_label.clone();
    provide_export(move || {
        let sized = has_sizes();
        let mut head = vec!["Group".to_string(), x_head.clone(), y_head.clone()];
        if sized {
            head.push(size_head.clone());
        }
        let body = groups.with(|g| {
            g.iter()
                .flat_map(|g| {
                    g.points.iter().map(move |p| {
                        let mut row = vec![g.name.clone(), fx(p.x), fy(p.y)];
                        if sized {
                            row.push(p.size.map(format_value).unwrap_or_default());
                        }
                        row
                    })
                })
                .collect::<Vec<_>>()
        });
        std::iter::once(head).chain(body).collect()
    });
    let colours = Signal::derive(move || (0..groups.with(Vec::len).max(1)).map(colour).collect::<Vec<_>>());
    let draw = Callback::new(move |f: crate::raster_canvas::Frame| layout.with(|l| raster::scatter(&l.dots, &f.colours, f.width, f.height, f.dpr)));
    let y_label_title = y_label.clone();

    view! {
        <figure
            data-scope="scatter-chart"
            data-part="root"
            style=move || gutter(&layout.with(|l| l.y_ticks.iter().map(|g| fy(g.value)).collect::<Vec<_>>()), 40.0)
            data-zoom=zoom.then_some("")
            data-settled=move || settled.get().then_some("")
        >
            {y_label_title.map(|t| view! { <div data-part="axis-title" data-axis="y" aria-hidden="true">{t}</div> })}
            <div data-part="plot" style=format!("height: {height}px;")>
                <div data-part="grid" aria-hidden="true">
                    {move || {
                        layout
                            .get()
                            .y_ticks
                            .into_iter()
                            .map(|g| {
                                view! {
                                    <div data-part="gridline" data-axis="y" style=format!("bottom: {};", pct(g.fraction))>
                                        <span data-part="tick">{fy(g.value)}</span>
                                    </div>
                                }
                            })
                            .collect_view()
                    }}
                    {move || {
                        layout
                            .get()
                            .x_ticks
                            .into_iter()
                            .map(|g| {
                                view! {
                                    <div data-part="gridline" data-axis="x" style=format!("left: {};", pct(g.fraction))>
                                        <span data-part="tick">{fx(g.value)}</span>
                                    </div>
                                }
                            })
                            .collect_view()
                    }}
                </div>
                <div data-part="annotations" aria-hidden="true">{move || annotation_views(placed.get())}</div>
                <div
                    data-part="overlay"
                    tabindex="0"
                    role="group"
                    aria-label=if zoom {
                        format!("{label}. Use the arrow keys to step through points, plus and minus to zoom, Shift and the arrow keys to pan.")
                    } else {
                        format!("{label}. Use the arrow keys to step through points.")
                    }
                    on:pointermove=move_to
                    on:pointerdown=move |ev| {
                        if zoom {
                            gesture_down(gesture, &ev);
                        }
                        move_to(ev);
                    }
                    on:pointerup=move |ev| gesture.update_value(|g| g.up(ev.pointer_id()))
                    on:pointercancel=move |ev| gesture.update_value(|g| g.up(ev.pointer_id()))
                    on:pointerleave=move |_| active.set(None)
                    on:wheel=move |ev| {
                        if let Some(next) = zoom.then(|| wheel_zoom(&ev, view.get_untracked(), Axes::XY, MIN_SPAN)).flatten() {
                            set_view(next);
                        }
                    }
                    on:blur=move |_| active.set(None)
                    on:keydown=move |ev| {
                        if let Some(next) = zoom.then(|| view.get_untracked().key(&ev.key(), ev.shift_key(), Axes::XY, MIN_SPAN)).flatten() {
                            ev.prevent_default();
                            set_view(next);
                            return;
                        }
                        if let Some(next) = order.with(|o| scatter::step_key(&ev.key(), active.get_untracked(), o)) {
                            ev.prevent_default();
                            active.set(next);
                        }
                    }
                >
                    <Show when=move || big.get()>
                        <RasterCanvas colours draw />
                    </Show>
                    {move || {
                        if big.get() {
                            return Vec::new();
                        }
                        layout
                            .get()
                            .dots
                            .into_iter()
                            .enumerate()
                            .map(|(i, d)| {
                                let diameter = d.size.map_or(10.0, |s| 8.0 + s * 32.0);
                                view! {
                                    <span
                                        data-part="dot"
                                        data-group=d.group
                                        data-bubble=d.size.map(|_| "")
                                        data-active=move || (active.get() == Some(i)).then_some("")
                                        style=format!(
                                            "left: {}; bottom: {}; --d: {diameter:.1}px; --i: {}; background: {};",
                                            pct(d.x),
                                            pct(d.y),
                                            i.min(40),
                                            colour(d.group),
                                        )
                                    ></span>
                                }
                            })
                            .collect::<Vec<_>>()
                    }}
                    {move || {
                        let i = active.get()?;
                        let d = layout.with(|l| l.dots.get(i).copied())?;
                        let p = point_of(d)?;
                        let name = groups.with(|g| g[d.group].name.clone());
                        let size_row = p
                            .size
                            .map(|s| {
                                view! {
                                    <li data-part="tooltip-row">
                                        <span data-part="tooltip-name">{size_label.clone()}</span>
                                        <span data-part="tooltip-value">{format_value(s)}</span>
                                    </li>
                                }
                            });
                        // Canvas dots have no element of their own, so the active one gets a marker.
                        let marker = big
                            .get_untracked()
                            .then(|| {
                                let diameter = d.size.map_or(10.0, |s| 8.0 + s * 32.0);
                                view! {
                                    <span
                                        data-part="dot"
                                        data-active=""
                                        style=format!(
                                            "left: {}; bottom: {}; --d: {diameter:.1}px; background: {}; animation: none;",
                                            pct(d.x),
                                            pct(d.y),
                                            colour(d.group),
                                        )
                                    ></span>
                                }
                            });
                        Some(
                            view! {
                                {marker}
                                <div data-part="crosshair" data-axis="x" style=format!("left: {};", pct(d.x))></div>
                                <div data-part="crosshair" data-axis="y" style=format!("bottom: {};", pct(d.y))></div>
                                <div
                                    data-part="tooltip"
                                    data-side={if d.x > 0.5 { "left" } else { "right" }}
                                    data-vertical={if d.y > 0.5 { "below" } else { "above" }}
                                    style=format!("left: {}; bottom: {};", pct(d.x), pct(d.y))
                                    aria-hidden="true"
                                >
                                    <div data-part="tooltip-label">
                                        <span data-part="swatch" style=format!("background: {};", colour(d.group))></span>
                                        {name}
                                    </div>
                                    <ul>
                                        <li data-part="tooltip-row">
                                            <span data-part="tooltip-name">{x_label.clone().unwrap_or_else(|| "x".into())}</span>
                                            <span data-part="tooltip-value">{fx(p.x)}</span>
                                        </li>
                                        <li data-part="tooltip-row">
                                            <span data-part="tooltip-name">{y_label.clone().unwrap_or_else(|| "y".into())}</span>
                                            <span data-part="tooltip-value">{fy(p.y)}</span>
                                        </li>
                                        {size_row}
                                    </ul>
                                </div>
                            },
                        )
                    }}
                </div>
                <Show when=move || zoom && !view.get().is_full()>
                    <button data-part="zoom-reset" type="button" on:click=move |_| view.set(View::FULL)>
                        "Reset zoom"
                    </button>
                </Show>
            </div>
            {x_label_title.map(|t| view! { <div data-part="axis-title" data-axis="x" aria-hidden="true">{t}</div> })}
            <Show when=move || groups.with(|g| g.len() > 1)>
                <ul data-part="legend" aria-hidden="true">
                    {move || {
                        groups
                            .get()
                            .into_iter()
                            .enumerate()
                            .map(|(i, g)| {
                                view! {
                                    <li data-part="legend-item" data-index=i>
                                        <span data-part="swatch" style=format!("background: {};", colour(i))></span>
                                        {g.name}
                                    </li>
                                }
                            })
                            .collect_view()
                    }}
                </ul>
            </Show>
            <span data-part="sr-live" aria-live="polite">{move || active.get().and_then(&describe).unwrap_or_default()}</span>
            <table data-part="sr-table">
                <caption>
                    {label.clone()}
                    {move || {
                        let n = groups.with(|g| g.iter().map(|g| g.points.len()).sum::<usize>());
                        (n > raster::TABLE_LIMIT).then(|| format!(" (first {} of {n} points)", raster::TABLE_LIMIT))
                    }}
                </caption>
                <thead>
                    <tr>
                        <th scope="col">"Group"</th>
                        <th scope="col">"X"</th>
                        <th scope="col">"Y"</th>
                        {move || has_sizes().then(|| view! { <th scope="col">{size_label_table.clone()}</th> })}
                    </tr>
                </thead>
                <tbody>
                    {move || {
                        let sized = has_sizes();
                        let rows: Vec<(String, Point)> = groups.with(|g| {
                            g.iter().flat_map(|g| g.points.iter().map(|p| (g.name.clone(), *p))).take(raster::TABLE_LIMIT).collect()
                        });
                        rows.into_iter()
                            .map(|(name, p)| {
                                view! {
                                    <tr>
                                        <th scope="row">{name}</th>
                                        <td>{fx(p.x)}</td>
                                        <td>{fy(p.y)}</td>
                                        {sized.then(|| view! { <td>{p.size.map(format_value).unwrap_or_default()}</td> })}
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
