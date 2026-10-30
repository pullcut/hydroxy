use hydroxy_leptos::json_inspector::JsonInspector;
use leptos::prelude::*;

// A webhook delivery as an API would log it, plus a long list of line items so the
// inspector groups them into ranges.
fn payload() -> String {
    let items = (1..=240)
        .map(|n| {
            format!(
                r#"{{"sku":"SKU-{n:04}","qty":{},"unit_price":{}.{:02},"backordered":{}}}"#,
                n % 4 + 1,
                n % 50 + 3,
                n * 7 % 100,
                n % 17 == 0
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!(
        r#"{{
  "id": "evt_1Q8xR2Lk",
  "type": "order.fulfilled",
  "created": 1729680312,
  "livemode": false,
  "data": {{
    "order": {{
      "number": "A-10482",
      "customer": {{ "name": "Grace Hopper", "email": "grace@example.com", "vip": true }},
      "shipping address": {{ "line1": "12 Harbour Row", "city": "Bristol", "postcode": "BS1 4RN", "country": "GB" }},
      "notes": null,
      "tags": ["priority", "gift-wrap", "repeat-customer"],
      "items": [{items}]
    }}
  }},
  "attempts": [
    {{ "at": "2024-10-23T10:45:12Z", "status": 500, "latency_ms": 1204 }},
    {{ "at": "2024-10-23T10:46:12Z", "status": 200, "latency_ms": 87.5 }}
  ]
}}"#
    )
}

#[component]
pub fn Demo() -> impl IntoView {
    view! {
        <div style="max-width: 720px;">
            <JsonInspector json=payload() expand_depth=3 label="Webhook payload" />
        </div>
    }
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "json-inspector",
    name: "JSON inspector",
    group: "Data",
    summary: "A collapsible JSON tree with values coloured by type. Search highlights matches and opens the way to them, any row copies its value or its path, and long arrays fold into ranges of 100 so huge payloads stay quick.",
    demo: || view! { <Demo /> }.into_any(),
    leptos: include_str!("json_inspector.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/json_inspector.rs"),
    parts: &[
        &["JsonInspector", "root", "Toolbar over the tree"],
        &["", "toolbar", "Search, match status and the expand-to-depth buttons"],
        &["", "status", "role=status: match count, or what was just copied"],
        &["", "tree", "<ul role=tree>. --ox-json-max-height caps its height (default 480px)"],
        &[
            "",
            "item",
            "<li role=treeitem>: aria-level, aria-setsize, aria-posinset, aria-expanded. data-state=open|closed, data-matched",
        ],
        &[
            "",
            "toggle / key / colon / value",
            "Chevron, key or index, separator, and the value. The value carries data-type: string, number, boolean, null, object, array or range",
        ],
        &["", "mark", "A search hit inside a key or value"],
        &["", "actions / copy", "Copy value and copy path buttons, shown on hover or focus"],
        &["", "error", "role=alert: the parse error with line and column"],
    ],
    keys: &[
        &["ArrowDown / ArrowUp", "Next / previous row"],
        &["ArrowRight", "Opens a closed node, or moves to its first child"],
        &["ArrowLeft", "Closes an open node, or moves to the parent"],
        &["Home / End", "First / last row"],
        &["Enter / Space", "Opens or closes the node"],
        &["*", "Opens every node beside the focused one"],
        &["Ctrl/⌘ + C", "Copies the focused row's value as JSON"],
        &["Ctrl/⌘ + Shift + C", "Copies the focused row's path, such as $.data.order.items[2].sku"],
        &["Enter / Shift + Enter", "In search: jump to the next / previous match"],
    ],
    props: &[
        &["json", "String", "Required. The JSON text. Invalid JSON shows the parse error"],
        &["expand_depth", "usize", "Levels open at start; the root is level 1. Default 2"],
        &["label", "String", "Accessible name for the tree. Default \"JSON\""],
    ],
    events: &[],
};
