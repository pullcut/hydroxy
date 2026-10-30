use hydroxy_leptos::icon::{Icon, IconName};
use hydroxy_leptos::input::Input;
use hydroxy_leptos::qr_code::{ErrorCorrection, QrCode};
use hydroxy_leptos::segmented::{Segmented, SegmentedItem, SegmentedOption};
use leptos::prelude::*;

#[component]
pub fn Demo() -> impl IntoView {
    let (ticket, set_ticket) = signal("https://tickets.hydroxy.dev/rustconf-2026/K7Q2-9XFD".to_string());
    let (level, set_level) = signal(Some("M".to_string()));
    let ecc = Signal::derive(move || {
        level.with(|l| ErrorCorrection::ALL.into_iter().find(|e| Some(e.as_str()) == l.as_deref()).unwrap_or_default())
    });

    let ecc_items: Vec<_> = ErrorCorrection::ALL.iter().map(|e| SegmentedOption::new(e.as_str())).collect();
    view! {
        <div style="display: flex; flex-wrap: wrap; gap: var(--space-9); align-items: flex-start;">
            // Live: edit the text or the error correction and the code redraws.
            <div style="display: grid; gap: var(--space-5); width: 280px;">
                <QrCode text=ticket ecc=ecc label="Ticket for RustConf 2026" download=true file_name="rustconf-ticket.svg" />
                <Input label="Ticket link" value=ticket on_input=move |t| set_ticket.set(t) />
                <Segmented
                    label="Error correction"
                    items=ecc_items
                    value=level
                    on_value_change=move |v| set_level.set(Some(v))
                >
                    <SegmentedItem value="L" label="L 7%" />
                    <SegmentedItem value="M" label="M 15%" />
                    <SegmentedItem value="Q" label="Q 25%" />
                    <SegmentedItem value="H" label="H 30%" />
                </Segmented>
            </div>

            // A Wi-Fi join code: phones offer to join the network when they scan it.
            <QrCode
                text="WIFI:T:WPA;S:Hydroxy Studio;P:copper-kettle-42;;"
                label="Join the Hydroxy Studio Wi-Fi"
                size=160
            />

            // A logo in the middle forces error correction H so the code still scans.
            <QrCode text="https://hydroxy.dev/docs" label="Hydroxy docs" size=160 color="var(--amethyst-800)">
                <span style="display: grid; place-items: center; width: 100%; height: 100%; border-radius: var(--radius-sm); background: var(--amethyst-600); color: #fff;">
                    <Icon name=IconName::FileText size=20 />
                </span>
            </QrCode>
        </div>
    }
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "qr-code",
    name: "QR code",
    group: "Display",
    summary: "A QR code encoded in Rust and drawn as one SVG path. Error correction L to H, an optional centre logo and an SVG download.",
    demo: || view! { <Demo /> }.into_any(),
    leptos: include_str!("qr_code.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/qr_code.rs"),
    parts: &[
        &["QrCode", "root", "Carries --qr-size, and --qr-fg / --qr-bg when colours are given"],
        &["QrCode", "frame", "Square box holding the code and the logo"],
        &["QrCode", "svg", "role=img with aria-label. One unit per module, quiet zone included"],
        &["QrCode", "background / modules", "The light square and every dark module as a single path"],
        &["QrCode", "logo", "Children, centred over a cleared square about a fifth of the code wide"],
        &["QrCode", "download", "A link to the code as a standalone SVG file"],
        &["QrCode", "error", "role=alert, in place of the code when the text is too long"],
    ],
    keys: &[],
    props: &[
        &["text", "String", "Required. What the code holds"],
        &["ecc", "ErrorCorrection", "Low, Medium (default), Quartile or High. A logo always uses High"],
        &["size", "u32", "Width and height in px. Default 192"],
        &["margin", "usize", "Quiet zone in modules. Default 4"],
        &["color", "String", "Module colour. Default var(--ink-900), dark in every theme"],
        &["background", "String", "Background colour. Default var(--paper-0)"],
        &["label", "String", "Accessible name. Default \"QR code: <text>\""],
        &["download", "bool", "Show a Download SVG link"],
        &["file_name", "String", "Download file name. Default qr-code.svg"],
        &["children", "View", "A logo for the centre"],
    ],
    events: &[],
};
