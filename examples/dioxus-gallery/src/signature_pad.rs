use dioxus::prelude::*;
use oxide_dioxus::signature_pad::{SignaturePad, SignatureValue};

#[component]
pub fn SignaturePadDemo() -> Element {
    let mut signed = use_signal(|| None::<SignatureValue>);
    let summary = match signed() {
        Some(SignatureValue::Drawn(d)) => format!("Drawn signature, {} bytes of path data", d.len()),
        Some(SignatureValue::Typed(name)) => format!("Typed signature: {name}"),
        None => "Not signed yet".to_string(),
    };

    rsx! {
        div { style: "display: grid; gap: var(--space-8); max-width: 520px;",
            // Draw with a mouse, pen or finger, or switch to Type for the keyboard.
            div { style: "display: grid; gap: var(--space-3);",
                strong { "Accept the 2027 service agreement" }
                SignaturePad { label: "Your signature", name: "signature", on_change: move |v| signed.set(v) }
                span { style: "color: var(--text-2);", "{summary}" }
            }

            // A countersignature already on file, shown read-only.
            div { style: "display: grid; gap: var(--space-3);",
                strong { "Countersigned by Northwind Ltd" }
                SignaturePad {
                    label: "Countersignature",
                    readonly: true,
                    default_value: SignatureValue::Typed("Maya Okafor".into()),
                }
            }
        }
    }
}
