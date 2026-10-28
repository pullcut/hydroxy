//! `ChartFrame`: the shared shell around a chart: title, description, legend, CSV
//! export, and the empty and loading states.

use dioxus::prelude::*;

use crate::button::ButtonSize;
use crate::icon::IconName;
use crate::icon_button::IconButton;
use crate::plot_kit::colour;

/// Reads the chart's `sr-table` rows, waits for the CSV text, then downloads it through a Blob URL.
const EXPORT: &str = r#"
const root = document.getElementById(ID);
const table = root && root.querySelector('[data-part="chart"] [data-part="sr-table"]');
dioxus.send(table ? [...table.querySelectorAll("tr")].map((tr) => [...tr.querySelectorAll("th, td")].map((c) => c.textContent)) : []);
const [name, text] = await dioxus.recv();
const url = URL.createObjectURL(new Blob([text], { type: "text/csv;charset=utf-8" }));
const link = document.createElement("a");
link.href = url;
link.download = name;
link.click();
URL.revokeObjectURL(url);
"#;

/// Set by a chart inside a [`ChartFrame`] whose table lists only some of its data, so
/// "Download CSV" still saves every row. Rows include the header row.
#[derive(Clone, Copy)]
pub(crate) struct ExportRows(pub(crate) CopyValue<Option<Callback<(), Vec<Vec<String>>>>>);

/// Hand the enclosing [`ChartFrame`], if any, the chart's full rows for CSV export. A hook:
/// call it unconditionally in the chart's body.
pub(crate) fn use_export(rows: impl FnMut(()) -> Vec<Vec<String>> + 'static) {
    let rows = use_callback(rows);
    use_hook(|| {
        if let Some(ExportRows(mut slot)) = try_consume_context::<ExportRows>() {
            slot.set(Some(rows));
        }
    });
}

/// A titled card section for any chart. While `loading` the chart is replaced by a
/// pulsing placeholder; when `empty` it is replaced by `empty_text`.
#[component]
pub fn ChartFrame(
    /// Heading, such as "Interviews read".
    title: Option<String>,
    /// One line under the title saying what the numbers are.
    description: Option<String>,
    /// Series names for a shared legend, in the categorical colour order. Leave empty when the chart has its own.
    #[props(default)]
    legend: Vec<String>,
    /// There is nothing to plot.
    #[props(default)]
    empty: bool,
    /// Data is on its way. Takes precedence over `empty`.
    #[props(default)]
    loading: bool,
    /// Message shown when empty.
    #[props(default = "No data for this period".to_string())]
    empty_text: String,
    /// Height in px of the placeholder shown while loading or empty.
    #[props(default = 180)]
    placeholder_height: u16,
    /// Show a "Download CSV" button in the header that saves the chart's data table,
    /// named after the title.
    #[props(default)]
    export: bool,
    /// Makes each legend entry a button that toggles it, calling this with the series name and
    /// whether it is now shown. The app hides or shows the series; the frame only dims the entry.
    on_series_toggle: Option<EventHandler<(String, bool)>>,
    /// The chart.
    children: Element,
) -> Element {
    let mut hidden = use_signal(Vec::<usize>::new);
    let id = use_hook(|| hydroxy_headless::next_id("chart-frame"));
    let state = if loading {
        "loading"
    } else if empty {
        "empty"
    } else {
        "ready"
    };
    let file = hydroxy_charts::csv::file_name(title.as_deref().unwrap_or("chart"));
    let script_id = id.clone();
    let full_rows = use_context_provider(|| ExportRows(CopyValue::new(None))).0;
    let download = move |_| {
        let (id, file) = (script_id.clone(), file.clone());
        let full = (*full_rows.read()).map(|rows| rows.call(()));
        spawn(async move {
            // `{id:?}` quotes the id as a JS string. Ids from `next_id` are plain ASCII.
            let mut js = document::eval(&EXPORT.replace("ID", &format!("{id:?}")));
            if let Ok(table) = js.recv::<Vec<Vec<String>>>().await {
                let rows = full.unwrap_or(table);
                let _ = js.send((file, hydroxy_charts::csv::to_csv(&rows)));
            }
        });
    };
    rsx! {
        section {
            id: "{id}",
            "data-scope": "chart-frame",
            "data-part": "root",
            "data-state": state,
            "aria-busy": loading.then_some("true"),
            if title.is_some() || description.is_some() || export {
                header { "data-part": "header",
                    if let Some(t) = title {
                        h3 { "data-part": "title", "{t}" }
                    }
                    if let Some(d) = description {
                        p { "data-part": "description", "{d}" }
                    }
                    if export {
                        div { "data-part": "actions",
                            IconButton { icon: IconName::Download, label: "Download CSV", size: ButtonSize::Sm, on_click: download }
                        }
                    }
                }
            }
            div { "data-part": "body",
                // The chart stays mounted and is only hidden, so toggling states never replays its entrance.
                div { "data-part": "chart", hidden: state != "ready", {children} }
                match state {
                    "loading" => rsx! {
                        div {
                            "data-part": "loading",
                            role: "status",
                            "aria-label": "Loading chart",
                            style: "min-height: {placeholder_height}px;",
                        }
                    },
                    "empty" => rsx! {
                        div { "data-part": "empty", style: "min-height: {placeholder_height}px;", "{empty_text}" }
                    },
                    _ => rsx! {},
                }
            }
            if !legend.is_empty() {
                ul { "data-part": "legend",
                    for (i , name) in legend.iter().enumerate() {
                        li { "data-part": "legend-item", "data-hidden": hidden.read().contains(&i).then_some(""),
                            span { "data-part": "swatch", style: "background: {colour(i)};" }
                            if let Some(cb) = on_series_toggle {
                                button {
                                    r#type: "button",
                                    "data-part": "legend-toggle",
                                    "aria-pressed": (!hidden.read().contains(&i)).to_string(),
                                    onclick: {
                                        let name = name.clone();
                                        move |_| {
                                            let shown = {
                                                let mut h = hidden.write();
                                                if let Some(at) = h.iter().position(|&x| x == i) {
                                                    h.remove(at);
                                                    true
                                                } else {
                                                    h.push(i);
                                                    false
                                                }
                                            };
                                            cb.call((name.clone(), shown));
                                        }
                                    },
                                    "{name}"
                                }
                            } else {
                                "{name}"
                            }
                        }
                    }
                }
            }
        }
    }
}
