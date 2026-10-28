//! `<PictogramChart>`: an isotype chart, each category's count drawn as a row of repeated
//! icons. Icon counts come from `hydroxy_charts::pictogram`. More than six categories fold
//! into "Other".

use hydroxy_charts::{fold_other, format_value, nav, pictogram, plot};
use leptos::prelude::*;

pub use hydroxy_charts::Datum;
pub use hydroxy_charts::pictogram::PictogramIcon;

/// Most icons a row draws; a smaller unit is raised to fit.
const MOST_ICONS: usize = 100;

fn row_colour(i: usize, len: usize, folded: bool) -> String {
    if folded && i + 1 == len { "var(--chart-muted)".into() } else { format!("var(--chart-{})", i.min(5) + 1) }
}

/// One row per category, its value drawn as repeated icons that each stand for `unit`
/// units; the last icon is cut to the remainder. A key under the rows says what one icon
/// is worth. Icons pop in row by row. Hover, touch or focus a row to read it; focus the
/// chart and use Up/Down to move between rows, Home/End to jump, Escape to clear.
#[allow(clippy::too_many_lines, reason = "one view! tree; splitting it only adds prop plumbing")]
#[allow(clippy::needless_pass_by_value, reason = "component props are owned")]
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
    let colour = move |i: usize| row_colour(i, parts.with(Vec::len), folded.get());
    let active = RwSignal::new(None::<usize>);
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
                on:pointerleave=move |_| active.set(None)
                on:blur=move |_| active.set(None)
                on:keydown=move |ev| {
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
                            let fill = colour(i);
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
                            let tip_value = fmt(d.value);
                            let tip_icons = format_value((d.value / unit).max(0.0));
                            let tip_label = d.label.clone();
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
                                                <div data-part="tooltip" data-anchored="" style="left: 0; top: 50%;" aria-hidden="true">
                                                    <div data-part="tooltip-label">
                                                        <span data-part="swatch" style=format!("background: {tip_fill};")></span>
                                                        {tip_label.clone()}
                                                    </div>
                                                    <ul>
                                                        <li data-part="tooltip-row">
                                                            <span data-part="tooltip-name">"Value"</span>
                                                            <span data-part="tooltip-value">{tip_value.clone()}</span>
                                                        </li>
                                                        <li data-part="tooltip-row">
                                                            <span data-part="tooltip-name">"Icons"</span>
                                                            <span data-part="tooltip-value">{tip_icons.clone()}</span>
                                                        </li>
                                                    </ul>
                                                </div>
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
                <caption>{label.clone()}</caption>
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
