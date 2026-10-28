//! `QrCode`: a QR code drawn as one SVG path. Children become a centred
//! logo (which forces error correction H). Optionally offers the code as an
//! SVG download.

use dioxus::prelude::*;
use hydroxy_headless::qr_code::{self as h, QrCode as Code};

pub use hydroxy_headless::qr_code::ErrorCorrection;

/// A QR code for `text`, sized in px with a quiet zone in modules.
#[component]
pub fn QrCode(
    /// What the code holds: a URL, a Wi-Fi join string, any text.
    text: ReadSignal<String>,
    /// Error correction. Default medium; a logo always uses high.
    #[props(default)]
    ecc: ReadSignal<ErrorCorrection>,
    /// Width and height in px. Default 192.
    #[props(default = h::DEFAULT_SIZE)]
    size: u32,
    /// Quiet zone in modules. Default 4, the standard's minimum.
    #[props(default = h::DEFAULT_MARGIN)]
    margin: usize,
    /// Module colour, any CSS colour. Default `var(--ink-900)`.
    #[props(default)]
    color: Option<String>,
    /// Background colour. Default `var(--paper-0)`.
    #[props(default)]
    background: Option<String>,
    /// Accessible name. Default `"QR code: <text>"`.
    #[props(default)]
    label: Option<String>,
    /// Show a "Download SVG" link.
    #[props(default)]
    download: bool,
    /// File name for the download. Default `qr-code.svg`.
    #[props(default = "qr-code.svg".to_string())]
    file_name: String,
    /// A logo for the centre, such as an icon or avatar.
    children: Element,
) -> Element {
    let logo = children != VNode::empty();
    let code = use_memo(move || Code::encode(&text.read(), h::effective_ecc(ecc(), logo)).ok());
    let style = h::root_style(size, color.as_deref(), background.as_deref());
    let aria = h::aria_label(&text.read(), label.as_deref());
    let code = code.read();

    rsx! {
        div { "data-scope": h::SCOPE, "data-part": "root", style,
            div { "data-scope": h::SCOPE, "data-part": "frame",
                if let Some(qr) = code.as_ref() {
                    svg {
                        "data-scope": h::SCOPE,
                        "data-part": "svg",
                        role: "img",
                        "aria-label": aria,
                        "viewBox": h::view_box(qr, margin),
                        width: "{size}",
                        height: "{size}",
                        "shape-rendering": "crispEdges",
                        rect { "data-scope": h::SCOPE, "data-part": "background", width: "100%", height: "100%" }
                        path { "data-scope": h::SCOPE, "data-part": "modules", d: h::path(qr, margin, logo) }
                    }
                    if logo {
                        div {
                            "data-scope": h::SCOPE,
                            "data-part": "logo",
                            "aria-hidden": "true",
                            style: h::logo_style(qr, margin),
                            {children}
                        }
                    }
                } else {
                    p { "data-scope": h::SCOPE, "data-part": "error", role: "alert", {h::TOO_LONG} }
                }
            }
            if download && let Some(qr) = code.as_ref() {
                a {
                    class: "ox-button",
                    "data-variant": "secondary",
                    "data-size": "sm",
                    "data-scope": h::SCOPE,
                    "data-part": "download",
                    href: h::data_url(&h::svg_document(qr, margin, color.as_deref(), background.as_deref())),
                    download: file_name,
                    "Download SVG"
                }
            }
        }
    }
}
