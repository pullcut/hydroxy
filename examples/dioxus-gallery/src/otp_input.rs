use dioxus::prelude::*;
use oxide_dioxus::otp_input::{OtpInput, OtpMode};

#[component]
pub fn OtpInputDemo() -> Element {
    let mut status = use_signal(|| "Enter the 6-digit code we sent you".to_string());
    let mut wrong = use_signal(|| false);

    rsx! {
        div { style: "display: grid; gap: var(--space-8);",
            div { style: "display: grid; gap: var(--space-3);",
                OtpInput {
                    label: "Verification code",
                    invalid: wrong(),
                    on_value_change: move |_| wrong.set(false),
                    on_complete: move |code: String| {
                        let ok = code == "123456";
                        wrong.set(!ok);
                        status.set(if ok { "Verified".into() } else { "That code didn't match. Try 123456".into() });
                    },
                }
                p { style: "margin: 0; font: var(--text-sm); color: var(--text-3);", "{status}" }
            }
            OtpInput { label: "Recovery key", length: 4, mode: OtpMode::Alphanumeric, name: "key" }
        }
    }
}
