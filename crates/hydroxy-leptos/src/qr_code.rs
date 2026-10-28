//! `<QrCode>`: a QR code drawn as one SVG path. Children become a centred
//! logo (which forces error correction H). Optionally offers the code as an
//! SVG download.

use hydroxy_headless::qr_code::{self as h, QrCode as Code};
use leptos::prelude::*;

pub use hydroxy_headless::qr_code::ErrorCorrection;

/// A QR code for `text`, sized in px with a quiet zone in modules.
#[component]
pub fn QrCode(
    /// What the code holds: a URL, a Wi-Fi join string, any text.
    #[prop(into)]
    text: Signal<String>,
    /// Error correction. Default medium; a logo always uses high.
    #[prop(optional, into)]
    ecc: Signal<ErrorCorrection>,
    /// Width and height in px. Default 192.
    #[prop(default = h::DEFAULT_SIZE)]
    size: u32,
    /// Quiet zone in modules. Default 4, the standard's minimum.
    #[prop(default = h::DEFAULT_MARGIN)]
    margin: usize,
    /// Module colour, any CSS colour. Default `var(--ink-900)`.
    #[prop(optional, into)]
    color: Option<String>,
    /// Background colour. Default `var(--paper-0)`.
    #[prop(optional, into)]
    background: Option<String>,
    /// Accessible name. Default `"QR code: <text>"`.
    #[prop(optional, into)]
    label: Option<String>,
    /// Show a "Download SVG" link.
    #[prop(optional)]
    download: bool,
    /// File name for the download. Default `qr-code.svg`.
    #[prop(optional, into)]
    file_name: Option<String>,
    /// A logo for the centre, such as an icon or avatar.
    #[prop(optional)]
    children: Option<Children>,
) -> impl IntoView {
    let logo = children.is_some();
    let code = Memo::new(move |_| text.with(|t| Code::encode(t, h::effective_ecc(ecc.get(), logo)).ok()));
    let style = h::root_style(size, color.as_deref(), background.as_deref());
    let file_name = file_name.unwrap_or_else(|| "qr-code.svg".into());
    let href = move || {
        code.with(|c| {
            c.as_ref().map(|qr| h::data_url(&h::svg_document(qr, margin, color.as_deref(), background.as_deref())))
        })
    };
    let aria = Signal::derive(move || text.with(|t| h::aria_label(t, label.as_deref())));

    view! {
        <div data-scope=h::SCOPE data-part="root" style=style>
            <div data-scope=h::SCOPE data-part="frame">
                <Show
                    when=move || code.with(Option::is_some)
                    fallback=|| view! { <p data-scope=h::SCOPE data-part="error" role="alert">{h::TOO_LONG}</p> }
                >
                    <svg
                        data-scope=h::SCOPE
                        data-part="svg"
                        role="img"
                        aria-label=aria
                        viewBox=move || code.with(|c| c.as_ref().map(|qr| h::view_box(qr, margin)))
                        width=size
                        height=size
                        shape-rendering="crispEdges"
                    >
                        <rect data-scope=h::SCOPE data-part="background" width="100%" height="100%" />
                        <path data-scope=h::SCOPE data-part="modules" d=move || code.with(|c| c.as_ref().map(|qr| h::path(qr, margin, logo))) />
                    </svg>
                </Show>
                {children.map(|logo_view| {
                    view! {
                        <div
                            data-scope=h::SCOPE
                            data-part="logo"
                            aria-hidden="true"
                            style=move || code.with(|c| c.as_ref().map(|qr| h::logo_style(qr, margin)))
                        >
                            {logo_view()}
                        </div>
                    }
                })}
            </div>
            {download.then(|| view! {
                <a
                    class="ox-button"
                    data-variant="secondary"
                    data-size="sm"
                    data-scope=h::SCOPE
                    data-part="download"
                    href=href
                    download=file_name
                    aria-disabled=move || code.with(Option::is_none).then_some("true")
                >
                    "Download SVG"
                </a>
            })}
        </div>
    }
}
