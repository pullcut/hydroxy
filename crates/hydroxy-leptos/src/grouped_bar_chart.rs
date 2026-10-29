//! `<GroupedBarChart>`: several series side by side in each category. Geometry comes from
//! `hydroxy_charts::grouped`; bars are positioned with CSS percentages.

use hydroxy_charts::event::Datum as EventDatum;
use hydroxy_charts::grouped;
use hydroxy_charts::nav::grid_key;
use hydroxy_charts::plot::{self, Key, Tip, TipRow, colour};
use leptos::prelude::*;

pub use hydroxy_charts::Orientation;
pub use hydroxy_charts::annotate::Annotation;
pub use hydroxy_charts::line::Series;

use crate::plot_kit::{Plot, formatter, placed};

/// Series side by side within each category, from a zero baseline. Hover or touch a bar to
/// read its whole group; focus the plot and use the arrow keys to move between bars
/// (along the categories, and across series), Home/End to jump, Escape to clear.
#[component]
pub fn GroupedBarChart(
    /// One label per category.
    #[prop(into)]
    categories: Signal<Vec<String>>,
    /// The series, one value per category, in categorical colour order. Past six, the
    /// smallest fold into "Other". NaN leaves an empty slot.
    #[prop(into)]
    series: Signal<Vec<Series>>,
    /// What the chart shows, such as "Signups by channel". Used as the accessible name.
    #[prop(into)]
    label: String,
    /// Plot height in px, excluding the axis labels and legend.
    #[prop(default = 240)]
    height: u16,
    /// Vertical (default) or horizontal bars. Horizontal suits long category names.
    #[prop(optional)]
    orientation: Orientation,
    /// Formats values for the axis, tooltip and table. Default: whole numbers, else one decimal place.
    #[prop(optional, into)]
    format: Option<Callback<f64, String>>,
    /// Reference lines, bands and labelled points, in data units (x is a category index).
    #[prop(optional, into)]
    annotations: Signal<Vec<Annotation>>,
    /// Called when the pointer or keyboard moves to a bar, and with `None` when it leaves.
    #[prop(optional, into)]
    on_hover: Option<Callback<Option<EventDatum>>>,
    /// Called on click, tap or Enter on a bar.
    #[prop(optional, into)]
    on_select: Option<Callback<EventDatum>>,
) -> impl IntoView {
    let fmt = formatter(format);
    let horizontal = orientation == Orientation::Horizontal;
    let series = Memo::new(move |_| series.with(|s| plot::fold_series(s, 6)));
    let count = move || categories.with(Vec::len);
    let layout = Memo::new(move |_| series.with(|s| annotations.with(|a| grouped::layout(s, count(), a))));
    // (series, category), matching nav::grid_key's (row, column).
    let active = RwSignal::new(None::<(usize, usize)>);
    let datum = Signal::derive(move || {
        let (s, c) = active.get()?;
        series.with(|all| categories.with(|n| EventDatum::from_series(all, s, c, n.get(c).map(String::as_str))))
    });

    let tip = Signal::derive(move || {
        let (s, c) = active.get()?;
        let title = categories.with(|n| n.get(c).cloned())?;
        let bar = layout.with(|l| l.bars.iter().find(|b| (b.series, b.category) == (s, c)).copied());
        let along = bar.map_or_else(|| plot::band_centre(c, count()), |b| b.centre);
        let value = bar.map_or_else(|| layout.with(|l| l.axis.base()), |b| b.to.max(b.from));
        let rows = series.with(|all| {
            all.iter()
                .enumerate()
                .map(|(i, x)| TipRow {
                    active: i == s,
                    ..TipRow::new(Some(colour(i)), x.name.clone(), plot::cell(x.values.get(c).copied(), &fmt))
                })
                .collect()
        });
        Some(if horizontal {
            Tip { x: value, y: 1.0 - along, title, rows }
        } else {
            Tip { x: along, y: value, title, rows }
        })
    });
    let on_point = Callback::new(move |p: Option<(f64, f64)>| {
        let next =
            p.and_then(|(x, y)| grouped::pick(if horizontal { 1.0 - y } else { x }, count(), series.with(Vec::len)));
        if active.get_untracked() != next {
            active.set(next);
        }
    });
    let on_key = Callback::new(move |key: String| {
        let key = if horizontal { plot::turn_key(&key) } else { &key };
        grid_key(key, active.get_untracked(), series.with(Vec::len), count()).map(|next| active.set(next)).is_some()
    });
    let ticks = Signal::derive(move || layout.with(|l| l.axis.ticks(&fmt)));
    let labels =
        Signal::derive(move || categories.with(|c| plot::category_ticks(c, if horizontal { usize::MAX } else { 8 })));
    let notes = placed(
        move |a| {
            layout.with(|l| plot::place(a, horizontal, |x| plot::category_at(x, count()), |v| l.axis.scale.map(v)))
        },
        annotations,
    );
    let legend = Signal::derive(move || {
        series.with(|s| {
            if s.len() > 1 {
                s.iter().enumerate().map(|(i, s)| Key::new(s.name.clone(), colour(i))).collect()
            } else {
                Vec::new()
            }
        })
    });
    let head = Signal::derive(move || {
        std::iter::once(String::new())
            .chain(series.with(|s| s.iter().map(|s| s.name.clone()).collect::<Vec<_>>()))
            .collect()
    });
    let rows = Signal::derive(move || {
        categories.with(|names| {
            series.with(|s| {
                names
                    .iter()
                    .enumerate()
                    .map(|(c, n)| {
                        std::iter::once(n.clone())
                            .chain(s.iter().map(|x| plot::cell(x.values.get(c).copied(), &fmt)))
                            .collect()
                    })
                    .collect()
            })
        })
    });
    let marks = move || {
        view! {
            <div data-part="bars">
                <For
                    each=move || layout.get().bars
                    key=|b| (b.series, b.category)
                    children=move |b| {
                        let key = (b.series, b.category);
                        let bar = Memo::new(move |_| layout.with(|l| l.bars.iter().find(|x| (x.series, x.category) == key).copied()).unwrap_or(b));
                        view! {
                            <div
                                data-part="bar"
                                data-index=b.series
                                data-negative=move || bar.with(|b| b.to < b.from).then_some("")
                                data-active=move || (active.get() == Some(key)).then_some("")
                                style=move || {
                                    let b = bar.get();
                                    format!(
                                        "--c: {:.4}; --w: {:.4}; --a: {:.4}; --b: {:.4}; --i: {}; background: {};",
                                        b.centre,
                                        b.width,
                                        b.from.min(b.to),
                                        b.from.max(b.to),
                                        b.category,
                                        colour(b.series),
                                    )
                                }
                            ></div>
                        }
                    }
                />
            </div>
        }
    };

    view! {
        <Plot
            scope="grouped-bar-chart"
            label
            hint="Use the arrow keys to move between bars."
            height
            horizontal
            ticks
            labels
            annotations=notes
            legend
            tip
            head
            rows
            on_key
            on_point
            marks
            datum=Some(datum)
            on_hover
            on_select
        />
    }
}
