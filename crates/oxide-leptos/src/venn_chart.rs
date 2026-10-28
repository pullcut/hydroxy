//! `<VennChart>`: two or three sets as circles whose areas match their sizes, with each
//! pairwise overlap solved to the size of the members the sets share. Layout comes from
//! `oxide_charts::venn`.

use leptos::prelude::*;
use oxide_charts::{chord, format_value, nav, venn};

use crate::sets_kit::SetsTip;

pub use oxide_charts::venn::Intersection;

/// An area-proportional Venn diagram. Every exclusive region carries its member count;
/// hover or touch a region to read it and its share. Focus the chart and use the arrow
/// keys to step through the regions, Home and End to jump, Escape to clear. Only the
/// three largest sets are drawn; for more, use `UpsetChart`.
#[allow(clippy::too_many_lines, reason = "one view! tree; splitting it only adds prop plumbing")]
#[component]
pub fn VennChart(
    /// Members per exclusive region: each entry counts the members in exactly those sets.
    #[prop(into)]
    data: Signal<Vec<Intersection>>,
    /// What the chart shows, such as "Customers by channel". Used as the accessible name.
    #[prop(into)]
    label: String,
    /// Drawing width in px; the chart scales down to fit its container.
    #[prop(default = 520)]
    width: u16,
    /// Drawing height in px.
    #[prop(default = 360)]
    height: u16,
    /// Formats counts for labels, the tooltip and the table. Default: whole numbers, else one decimal place.
    #[prop(optional, into)]
    format: Option<Callback<f64, String>>,
) -> impl IntoView {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.run(v));
    let (box_w, box_h) = (f64::from(width), f64::from(height));
    let layout = Memo::new(move |_| data.with(|d| venn::layout(d, box_w, box_h)));
    // Keyboard steps through the regions that have room on the page.
    let shown = Memo::new(move |_| layout.with(|l| (0..l.regions.len()).filter(|&i| l.regions[i].label.is_some()).collect::<Vec<_>>()));
    let active = RwSignal::new(None::<usize>);
    let pick = move |i: usize| {
        if active.get_untracked() != Some(i) {
            active.set(Some(i));
        }
    };
    let in_active = move |set: usize| active.get().is_some_and(|a| layout.with(|l| l.regions.get(a).is_some_and(|r| r.sets.contains(&set))));
    let tip = move || active.get().and_then(|i| layout.with(|l| l.tip(i, &fmt)));
    let group_label = format!("{label}. Use the arrow keys to step through the regions.");
    let caption = label;

    view! {
        <figure data-scope="venn-chart" data-part="root" data-active=move || active.get().map(|_| "") style=format!("max-width: {width}px;")>
            <div data-part="plot">
                <svg
                    data-part="marks"
                    viewBox=format!("0 0 {width} {height}")
                    tabindex="0"
                    role="group"
                    aria-label=group_label
                    on:pointerleave=move |_| active.set(None)
                    on:blur=move |_| active.set(None)
                    on:keydown=move |ev| {
                        let list = shown.get_untracked();
                        let at = active.get_untracked().and_then(|a| list.iter().position(|&i| i == a));
                        if let Some(next) = nav::list_key(&ev.key(), at, list.len()) {
                            ev.prevent_default();
                            active.set(next.map(|k| list[k]));
                        }
                    }
                >
                    {move || {
                        let l = layout.get();
                        let n = l.circles.len();
                        let circles = l
                            .circles
                            .iter()
                            .enumerate()
                            .map(|(i, c)| {
                                view! {
                                    <circle
                                        data-part="circle"
                                        data-highlighted=move || in_active(i).then_some("")
                                        cx=c.cx
                                        cy=c.cy
                                        r=c.r
                                        fill=chord::colour(i, n)
                                        style=format!("--i: {i};")
                                    ></circle>
                                }
                            })
                            .collect_view();
                        let regions = l
                            .regions
                            .iter()
                            .enumerate()
                            .filter(|(_, r)| !r.path.is_empty())
                            .map(|(k, r)| {
                                view! {
                                    <path
                                        data-part="region"
                                        data-active=move || (active.get() == Some(k)).then_some("")
                                        d=r.path.clone()
                                        fill-rule="evenodd"
                                        on:pointerenter=move |_| pick(k)
                                        on:pointerdown=move |_| pick(k)
                                    ></path>
                                }
                            })
                            .collect_view();
                        let values = l
                            .regions
                            .iter()
                            .enumerate()
                            .filter_map(|(k, r)| {
                                let (x, y) = r.label?;
                                Some(view! {
                                    <text
                                        data-part="region-value"
                                        data-active=move || (active.get() == Some(k)).then_some("")
                                        data-empty=(r.count <= 0.0).then_some("")
                                        x=x
                                        y=y
                                        text-anchor="middle"
                                        dominant-baseline="central"
                                    >
                                        {fmt(r.count)}
                                    </text>
                                })
                            })
                            .collect_view();
                        let names = l
                            .circles
                            .iter()
                            .map(|c| {
                                view! {
                                    <text data-part="set-name" x=c.label.0 y=c.label.1 text-anchor=c.label.2 dominant-baseline="central">
                                        {c.name.clone()}
                                        <tspan data-part="set-size" dx="6">{fmt(c.size)}</tspan>
                                    </text>
                                }
                            })
                            .collect_view();
                        view! {
                            <g data-part="circles">{circles}</g>
                            <g data-part="regions">{regions}</g>
                            <g data-part="labels">{values}{names}</g>
                        }
                    }}
                </svg>
                {move || tip().map(|tip| view! { <SetsTip tip=tip /> })}
            </div>
            <span data-part="sr-live" aria-live="polite">{move || tip().map(|t| t.announce()).unwrap_or_default()}</span>
            <table data-part="sr-table">
                <caption>{caption}</caption>
                <thead>
                    <tr>
                        <th scope="col">"Region"</th>
                        <th scope="col">"Members"</th>
                    </tr>
                </thead>
                <tbody>
                    {move || {
                        layout
                            .with(|l| (0..l.regions.len()).map(|k| (l.region_name(k), l.regions[k].count)).collect::<Vec<_>>())
                            .into_iter()
                            .map(|(name, count)| {
                                view! {
                                    <tr>
                                        <th scope="row">{name}</th>
                                        <td>{fmt(count)}</td>
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
