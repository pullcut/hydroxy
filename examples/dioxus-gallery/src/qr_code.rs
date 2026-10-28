use dioxus::prelude::*;
use hydroxy_dioxus::icon::{Icon, IconName};
use hydroxy_dioxus::input::Input;
use hydroxy_dioxus::qr_code::{ErrorCorrection, QrCode};
use hydroxy_dioxus::segmented::{Segmented, SegmentedItem, SegmentedOption};

#[component]
pub fn QrCodeDemo() -> Element {
    let mut ticket = use_signal(|| "https://tickets.hydroxy.dev/rustconf-2026/K7Q2-9XFD".to_string());
    let mut level = use_signal(|| Some("M".to_string()));
    let ecc = use_memo(move || {
        level.with(|l| ErrorCorrection::ALL.into_iter().find(|e| Some(e.as_str()) == l.as_deref()).unwrap_or_default())
    });

    rsx! {
        div { style: "display: flex; flex-wrap: wrap; gap: var(--space-9); align-items: flex-start;",
            // Live: edit the text or the error correction and the code redraws.
            div { style: "display: grid; gap: var(--space-5); width: 280px;",
                QrCode { text: ticket(), ecc: ecc(), label: "Ticket for RustConf 2026", download: true, file_name: "rustconf-ticket.svg" }
                Input { label: "Ticket link", value: ticket(), on_input: move |t| ticket.set(t) }
                Segmented {
                    label: "Error correction",
                    items: ErrorCorrection::ALL.iter().map(|e| SegmentedOption::new(e.as_str())).collect::<Vec<_>>(),
                    value: level(),
                    on_value_change: move |v| level.set(Some(v)),
                    SegmentedItem { value: "L", label: "L 7%" }
                    SegmentedItem { value: "M", label: "M 15%" }
                    SegmentedItem { value: "Q", label: "Q 25%" }
                    SegmentedItem { value: "H", label: "H 30%" }
                }
            }

            // A Wi-Fi join code: phones offer to join the network when they scan it.
            QrCode {
                text: "WIFI:T:WPA;S:Hydroxy Studio;P:copper-kettle-42;;",
                label: "Join the Hydroxy Studio Wi-Fi",
                size: 160,
            }

            // A logo in the middle forces error correction H so the code still scans.
            QrCode { text: "https://hydroxy.dev/docs", label: "Hydroxy docs", size: 160, color: "var(--amethyst-800)",
                span { style: "display: grid; place-items: center; width: 100%; height: 100%; border-radius: var(--radius-sm); background: var(--amethyst-600); color: #fff;",
                    Icon { name: IconName::FileText, size: 20 }
                }
            }
        }
    }
}
