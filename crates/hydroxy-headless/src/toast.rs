//! Toast connector. Anatomy: region, toast, icon, body, title, description,
//! action, close, timer. Every part carries `data-scope="toast"` and its `data-part`.
//!
//! The region is an always-shown `popover="manual"` with `aria-live="polite"`.
//! Each toast's lifetime is the CSS animation on its `timer` part (duration from
//! [`ToastAttrs::timer_style`]); the adapter dismisses the toast on that
//! element's `animationend`. Hover and focus pause the animation in CSS.

pub use hydroxy_machines::toast::*;

/// The `data-scope` value on every toast part.
pub const SCOPE: &str = "toast";

/// Attributes for one toast.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToastAttrs {
    /// `role`: `"alert"` for danger (assertive), none otherwise (the region is polite).
    pub role: Option<&'static str>,
    /// `data-tone`.
    pub data_tone: &'static str,
    /// `data-state`: `"open"` or `"leaving"`.
    pub data_state: &'static str,
    /// `style` for the timer part, or `None` for a toast that stays until dismissed.
    pub timer_style: Option<String>,
}

/// Attributes for `toast`.
#[must_use]
pub fn toast_attrs(toast: &Toast) -> ToastAttrs {
    ToastAttrs {
        role: (toast.options.tone == ToastTone::Danger).then_some("alert"),
        data_tone: toast.options.tone.as_str(),
        data_state: if toast.leaving { "leaving" } else { "open" },
        timer_style: toast.options.duration_ms.map(|ms| format!("--ox-toast-duration: {ms}ms;")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn toast(options: ToastOptions) -> Toast {
        Toast { id: 0, options, leaving: false }
    }

    #[test]
    fn danger_is_an_alert_and_others_rely_on_the_polite_region() {
        assert_eq!(toast_attrs(&toast(ToastOptions::new("x").tone(ToastTone::Danger))).role, Some("alert"));
        assert_eq!(toast_attrs(&toast(ToastOptions::new("x"))).role, None);
    }

    #[test]
    fn timer_style_carries_duration_and_sticky_has_none() {
        let a = toast_attrs(&toast(ToastOptions::new("x").duration_ms(Some(2500))));
        assert_eq!(a.timer_style.as_deref(), Some("--ox-toast-duration: 2500ms;"));
        assert_eq!(toast_attrs(&toast(ToastOptions::new("x").duration_ms(None))).timer_style, None);
    }
}
