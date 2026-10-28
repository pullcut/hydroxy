//! `<ChartFrame>`: the shared shell around a chart, with a title, description, legend,
//! CSV export, and the empty and loading states.

use leptos::either::EitherOf3;
use leptos::prelude::*;
use leptos::wasm_bindgen::{JsCast, JsValue};
use web_sys::js_sys;

use crate::button::ButtonSize;
use crate::icon::IconName;
use crate::icon_button::IconButton;
use crate::plot_kit::colour;

/// Set by a chart inside a [`ChartFrame`] whose table lists only some of its data, so
/// "Download CSV" still saves every row. Rows include the header row.
#[derive(Clone, Copy)]
pub(crate) struct ExportRows(pub StoredValue<Option<Callback<(), Vec<Vec<String>>>>>);

/// Hand the enclosing [`ChartFrame`], if any, the chart's full rows for CSV export.
pub(crate) fn provide_export(rows: impl Fn() -> Vec<Vec<String>> + Send + Sync + 'static) {
    if let Some(ExportRows(slot)) = use_context::<ExportRows>() {
        slot.set_value(Some(Callback::new(move |()| rows())));
    }
}

/// The text of every row of the chart's `sr-table` inside `root`.
fn table_rows(root: &web_sys::Element) -> Option<Vec<Vec<String>>> {
    let table = root.query_selector(r#"[data-part="chart"] [data-part="sr-table"]"#).ok()??;
    let all = |el: &web_sys::Element, sel: &str| -> Vec<web_sys::Element> {
        let Ok(list) = el.query_selector_all(sel) else { return Vec::new() };
        (0..list.length()).filter_map(|i| list.item(i)?.dyn_into().ok()).collect()
    };
    Some(
        all(&table, "tr")
            .iter()
            .map(|tr| all(tr, "th, td").iter().map(|c| c.text_content().unwrap_or_default()).collect())
            .collect(),
    )
}

/// Hand the browser `text` as a file download called `name`, through a Blob URL.
fn save(name: &str, text: &str) -> Option<()> {
    let options = web_sys::BlobPropertyBag::new();
    options.set_type("text/csv;charset=utf-8");
    let parts = js_sys::Array::of1(&JsValue::from_str(text));
    let blob = web_sys::Blob::new_with_str_sequence_and_options(&parts, &options).ok()?;
    let url = web_sys::Url::create_object_url_with_blob(&blob).ok()?;
    let link: web_sys::HtmlAnchorElement = document().create_element("a").ok()?.dyn_into().ok()?;
    link.set_href(&url);
    link.set_download(name);
    link.click();
    web_sys::Url::revoke_object_url(&url).ok()
}

/// A titled card section for any chart. While `loading` the chart is replaced by a
/// pulsing placeholder; when `empty` it is replaced by `empty_text`.
#[component]
pub fn ChartFrame(
    /// Heading, such as "Interviews read".
    #[prop(optional, into)]
    title: Option<String>,
    /// One line under the title saying what the numbers are.
    #[prop(optional, into)]
    description: Option<String>,
    /// Series names for a shared legend, in the categorical colour order. Leave empty when the chart has its own.
    #[prop(optional, into)]
    legend: Vec<String>,
    /// There is nothing to plot.
    #[prop(optional, into)]
    empty: Signal<bool>,
    /// Data is on its way. Takes precedence over `empty`.
    #[prop(optional, into)]
    loading: Signal<bool>,
    /// Message shown when empty.
    #[prop(default = "No data for this period".into(), into)]
    empty_text: String,
    /// Height in px of the placeholder shown while loading or empty.
    #[prop(default = 180)]
    placeholder_height: u16,
    /// Show a "Download CSV" button in the header that saves the chart's data table,
    /// named after the title.
    #[prop(optional)]
    export: bool,
    /// The chart.
    children: Children,
) -> impl IntoView {
    let state = move || {
        if loading.get() {
            "loading"
        } else if empty.get() {
            "empty"
        } else {
            "ready"
        }
    };
    let has_head = title.is_some() || description.is_some() || export;
    let file = hydroxy_charts::csv::file_name(title.as_deref().unwrap_or("chart"));
    let root = NodeRef::<leptos::html::Section>::new();
    let full_rows = StoredValue::new(None);
    provide_context(ExportRows(full_rows));
    let download = move |_| {
        let full = full_rows.get_value().map(|rows: Callback<(), Vec<Vec<String>>>| rows.run(()));
        if let Some(rows) = full.or_else(|| root.get_untracked().and_then(|el| table_rows(&el))) {
            let _ = save(&file, &hydroxy_charts::csv::to_csv(&rows));
        }
    };
    let legend_items = (!legend.is_empty()).then(|| {
        legend
            .into_iter()
            .enumerate()
            .map(|(i, name)| {
                view! {
                    <li data-part="legend-item">
                        <span data-part="swatch" style=format!("background: {};", colour(i))></span>
                        {name}
                    </li>
                }
            })
            .collect_view()
    });
    let height = format!("min-height: {placeholder_height}px;");

    view! {
        <section
            node_ref=root
            data-scope="chart-frame"
            data-part="root"
            data-state=state
            aria-busy=move || (state() == "loading").then_some("true")
        >
            {has_head
                .then(|| {
                    view! {
                        <header data-part="header">
                            {title.map(|t| view! { <h3 data-part="title">{t}</h3> })}
                            {description.map(|d| view! { <p data-part="description">{d}</p> })}
                            {export
                                .then(|| {
                                    view! {
                                        <div data-part="actions">
                                            <IconButton icon=IconName::Download label="Download CSV" size=ButtonSize::Sm on_click=download />
                                        </div>
                                    }
                                })}
                        </header>
                    }
                })}
            <div data-part="body">
                // The chart stays mounted and is only hidden, so toggling states never replays its entrance.
                <div data-part="chart" hidden=move || state() != "ready">{children()}</div>
                {move || match state() {
                    "loading" => {
                        EitherOf3::A(view! {
                            <div data-part="loading" role="status" aria-label="Loading chart" style=height.clone()></div>
                        })
                    }
                    "empty" => EitherOf3::B(view! { <div data-part="empty" style=height.clone()>{empty_text.clone()}</div> }),
                    _ => EitherOf3::C(()),
                }}
            </div>
            {legend_items.map(|items| view! { <ul data-part="legend">{items}</ul> })}
        </section>
    }
}
