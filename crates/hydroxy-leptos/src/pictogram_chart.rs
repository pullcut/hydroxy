//! `<PictogramChart>`: an isotype chart, each category's count drawn as a row of repeated
//! icons. Icon counts come from `hydroxy_charts::pictogram`. More than six categories fold
//! into "Other".

use hydroxy_charts::event::Datum as EventDatum;
use hydroxy_charts::plot::{Tip, TipRow};
use hydroxy_charts::{fold_other, format_value, nav, pictogram, plot};
use leptos::prelude::*;

use crate::callbacks::{select_active, watch_hover};
use crate::plot_kit::{ChartTip, colour, colour_or_muted};

pub use hydroxy_charts::Datum;
pub use hydroxy_charts::pictogram::PictogramIcon;

/// Most icons a row draws; a smaller unit is raised to fit.
const MOST_ICONS: usize = 100;

/// One row per category, its value drawn as repeated icons that each stand for `unit`
/// units; the last icon is cut to the remainder. A key under the rows says what one icon
/// is worth. Icons pop in row by row. Hover, touch or focus a row to read it; focus the
/// chart and use Up/Down to move between rows, Home/End to jump, Escape to clear.
#[component]
pub fn PictogramChart(
    /// The categories, top to bottom, in categorical colour order.
    #[prop(into)]
    data: Signal<Vec<Datum>>,
    /// What the chart shows, such as "Visitors by region". Used as the accessible name.
    #[prop(into)]
    label: String,
    /// What one icon stands for. Default: a 1, 2 or 5 × 10ⁿ unit that fits the largest
    /// row in `columns` icons. A unit that would need more than 100 icons a row is raised.
    #[prop(optional, into)]
    unit: MaybeProp<f64>,
    /// What is counted, for the key, such as "people". Default: no noun.
    #[prop(optional, into)]
    unit_label: Option<String>,
    /// The icon: `Person` (default), `Square`, `Circle` or `Custom(path)` in a 24×24 box.
    #[prop(optional)]
    icon: PictogramIcon,
    /// Icons per line before a row wraps.
    #[prop(default = 20)]
    columns: usize,
    /// Icon size in px.
    #[prop(default = 20)]
    icon_size: u16,
    /// Formats values for the rows, key, tooltip and table.
    #[prop(optional, into)]
    format: Option<Callback<f64, String>>,
    /// Called with the row under the pointer or keyboard focus, and `None` when it clears.
    #[prop(optional)]
    on_hover: Option<Callback<Option<EventDatum>>>,
    /// Called with the row when it is clicked, tapped or Enter is pressed on it.
    #[prop(optional)]
    on_select: Option<Callback<EventDatum>>,
) -> impl IntoView {
    let fmt = move |v: f64| format.map_or_else(|| format_value(v), |f| f.run(v));
    let columns = columns.max(1);
    let path = icon.path().to_string();
    let parts = Memo::new(move |_| data.with(|d| fold_other(d, 6)));
    let folded = Memo::new(move |_| data.with(Vec::len) > 6);
    let each = Memo::new(move |_| {
        parts.with(|p| {
            let values: Vec<f64> = p.iter().map(|d| d.value).collect();
            let fit =
                pictogram::auto_unit(values.iter().copied().filter(|v| v.is_finite()).fold(0.0, f64::max), columns);
            pictogram::unit_for(&values, Some(unit.get().unwrap_or(fit)), MOST_ICONS)
        })
    });
    let row_colour = move |i: usize| {
        if folded.get() { colour_or_muted(i, parts.with(Vec::len).saturating_sub(1)) } else { colour(i) }
    };
    let active = RwSignal::new(None::<usize>);
    let datum = Signal::derive(move || {
        let i = active.get()?;
        parts.with(|p| p.get(i).map(|d| EventDatum::category(i, d.label.clone(), d.value)))
    });
    watch_hover(Some(datum), on_hover);
    let noun = unit_label.map(|n| format!(" {n}")).unwrap_or_default();
    let key_noun = noun.clone();
    let describe = move |i: usize| {
        parts.with(|p| {
            p.get(i).map(|d| {
                format!("{}: {}{noun}, {} icons", d.label, fmt(d.value), format_value((d.value / each.get()).max(0.0)))
            })
        })
    };
    let key_path = path.clone();
    let group_label = format!("{label}. Use the up and down arrow keys to move between rows.");

    view! {
        <figure
            data-scope="pictogram-chart"
            data-part="root"
            data-active=move || active.get().map(|_| "")
            style=format!("--ox-icon: {icon_size}px; --ox-cols: {columns};")
        >
            <div
                data-part="plot"
                tabindex="0"
                role="group"
                aria-label=group_label
                on:click=move |_| select_active(Some(datum), on_select)
                on:pointerleave=move |_| active.set(None)
                on:blur=move |_| active.set(None)
                on:keydown=move |ev| {
                    if ev.key() == "Enter" {
                        select_active(Some(datum), on_select);
                    }
                    if let Some(next) = nav::list_key(plot::turn_key(&ev.key()), active.get_untracked(), parts.with(Vec::len)) {
                        ev.prevent_default();
                        active.set(next);
                    }
                }
            >
                {move || {
                    let unit = each.get();
                    parts
                        .get()
                        .into_iter()
                        .enumerate()
                        .map(|(i, d)| {
                            let fill = row_colour(i);
                            let icons = pictogram::icons(d.value, unit)
                                .into_iter()
                                .enumerate()
                                .map(|(j, f)| {
                                    let path = path.clone();
                                    view! {
                                        <span data-part="icon" data-partial=(f < 1.0).then_some("") style=format!("--i: {j}; --f: {f:.4};")>
                                            <svg data-part="icon-ghost" viewBox="0 0 24 24" aria-hidden="true"><path d=path.clone()></path></svg>
                                            <svg data-part="icon-fill" viewBox="0 0 24 24" aria-hidden="true"><path d=path></path></svg>
                                        </span>
                                    }
                                })
                                .collect_view();
                            let tip = Tip {
                                x: 0.0,
                                y: 0.5,
                                title: d.label.clone(),
                                rows: vec![
                                    TipRow::new(None, "Value", fmt(d.value)),
                                    TipRow::new(None, "Icons", format_value((d.value / unit).max(0.0))),
                                ],
                            };
                            let tip_fill = fill.clone();
                            view! {
                                <div
                                    data-part="row"
                                    data-index=i
                                    data-active=move || (active.get() == Some(i)).then_some("")
                                    style=format!("--r: {i}; --ox-fill: {fill};")
                                    on:pointerenter=move |_| active.set(Some(i))
                                    on:pointerdown=move |_| active.set(Some(i))
                                >
                                    <span data-part="row-label" title=d.label.clone()>{d.label.clone()}</span>
                                    <span data-part="icons">
                                        {icons}
                                        <span data-part="anchor">
                                            <Show when=move || active.get() == Some(i)>
                                                <ChartTip
                                                    tip=tip.clone()
                                                    swatch=tip_fill.clone()
                                                    anchored=true
                                                    style="left: 0; top: 50%;".to_string()
                                                />
                                            </Show>
                                        </span>
                                    </span>
                                    <span data-part="row-value">{fmt(d.value)}</span>
                                </div>
                            }
                        })
                        .collect_view()
                }}
            </div>
            <p data-part="key" aria-hidden="true">
                <svg data-part="key-icon" viewBox="0 0 24 24"><path d=key_path></path></svg>
                {move || format!("= {}{key_noun}", fmt(each.get()))}
            </p>
            <span data-part="sr-live" aria-live="polite">{move || active.get().and_then(&describe).unwrap_or_default()}</span>
            <table data-part="sr-table">
                <caption>{label}</caption>
                <thead>
                    <tr>
                        <th scope="col">"Category"</th>
                        <th scope="col">"Value"</th>
                        <th scope="col">"Icons"</th>
                    </tr>
                </thead>
                <tbody>
                    {move || {
                        let unit = each.get();
                        parts
                            .get()
                            .into_iter()
                            .map(|d| {
                                view! {
                                    <tr>
                                        <th scope="row">{d.label}</th>
                                        <td>{fmt(d.value)}</td>
                                        <td>{format_value((d.value / unit).max(0.0))}</td>
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
