use hydroxy_leptos::signature_pad::{SignaturePad, SignatureValue};
use leptos::prelude::*;

pub fn demo() -> AnyView {
    let (signed, set_signed) = signal(None::<SignatureValue>);
    let summary = move || match signed.get() {
        Some(SignatureValue::Drawn(d)) => format!("Drawn signature, {} bytes of path data", d.len()),
        Some(SignatureValue::Typed(name)) => format!("Typed signature: {name}"),
        None => "Not signed yet".to_string(),
    };

    view! {
        <div style="display: grid; gap: var(--space-8); max-width: 520px;">
            // Draw with a mouse, pen or finger, or switch to Type for the keyboard.
            <div style="display: grid; gap: var(--space-3);">
                <strong>"Accept the 2027 service agreement"</strong>
                <SignaturePad label="Your signature" name="signature" on_change=move |v| set_signed.set(v) />
                <span style="color: var(--text-2);">{summary}</span>
            </div>

            // A countersignature already on file, shown read-only.
            <div style="display: grid; gap: var(--space-3);">
                <strong>"Countersigned by Northwind Ltd"</strong>
                <SignaturePad
                    label="Countersignature"
                    readonly=true
                    default_value=SignatureValue::Typed("Maya Okafor".into())
                />
            </div>
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "signature-pad",
    name: "Signature pad",
    group: "Forms",
    summary: "Draw a signature with smoothed, pressure-like strokes, or type a name as the keyboard alternative. Exports SVG path data.",
    demo,
    leptos: include_str!("signature_pad.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/signature_pad.rs"),
    parts: &[
        &["SignaturePad", "root", "role=group with the label. data-mode, data-empty, data-readonly, data-disabled"],
        &["SignaturePad", "modes / mode", "Draw and Type toggle buttons with aria-pressed"],
        &["SignaturePad", "surface", "The drawing <svg>, role=img, 600 × 200 viewBox"],
        &["SignaturePad", "guide / placeholder", "The dashed signing line and the empty-state hint"],
        &["SignaturePad", "stroke", "One filled <path> per stroke; data-live on the one being drawn"],
        &[
            "SignaturePad",
            "typed / typed-input / typed-preview",
            "The keyboard alternative: a name field and its script rendering",
        ],
        &["SignaturePad", "actions / undo / clear", "Remove the last stroke, or everything"],
        &["SignaturePad", "live", "Polite live region: empty, how many strokes, or the typed name"],
    ],
    keys: &[
        &["Tab", "Reach the Draw / Type toggle, the name field and the buttons"],
        &["Ctrl/Cmd + Z", "Undo the last stroke"],
        &["Enter / Space (mode)", "Switch between drawing and typing"],
    ],
    props: &[
        &["label", "String", "Required. Accessible name"],
        &["default_value", "Option<SignatureValue>", "Drawn(path data) or Typed(name) to start with"],
        &["disabled", "bool", "Block changes and dim the pad"],
        &["readonly", "bool", "Show the signature without controls"],
        &["name", "String", "Form field name; submits drawn:<path> or typed:<name>"],
        &[
            "on_change",
            "Callback<Option<SignatureValue>>",
            "Called after each change; None when blank. Pass the value to to_svg for a file",
        ],
    ],
};
