use dioxus::prelude::*;
use oxide_dioxus::json_inspector::JsonInspector;

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
pub fn JsonInspectorDemo() -> Element {
    let json = use_signal(payload);
    rsx! {
        div { style: "max-width: 720px;",
            JsonInspector { json, expand_depth: 3, label: "Webhook payload" }
        }
    }
}
