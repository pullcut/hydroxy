//! `<Slider>`: a native `<input type="range">` with a label row and a value
//! readout. The native element gives keyboard support, `aria-valuenow` and form
//! participation; oxide only styles it.

use leptos::prelude::*;

/// The `data-scope` value on every slider part.
pub const SCOPE: &str = "slider";

/// How far `value` is from `min` to `max`, as a percentage for the filled track.
fn fill_percent(value: f64, min: f64, max: f64) -> f64 {
    if max > min { ((value - min) / (max - min) * 100.0).clamp(0.0, 100.0) } else { 0.0 }
}

/// A labelled range control.
#[component]
pub fn Slider(
    /// Visible label. Also the accessible name.
    #[prop(into)]
    label: String,
    /// Lowest value.
    #[prop(default = 0.0)]
    min: f64,
    /// Highest value.
    #[prop(default = 100.0)]
    max: f64,
    /// Step between values.
    #[prop(default = 1.0)]
    step: f64,
    /// Controlled value. Pair with `on_input`.
    #[prop(optional, into)]
    value: MaybeProp<f64>,
    /// Initial value when uncontrolled. Defaults to `min`.
    #[prop(optional, into)]
    default_value: Option<f64>,
    /// Called with the new value as the thumb moves.
    #[prop(optional, into)]
    on_input: Option<Callback<f64>>,
    /// Turns the value into the readout text, such as `"£1,200"`. Defaults to the number.
    #[prop(optional, into)]
    format: Option<Callback<f64, String>>,
    /// Block changes.
    #[prop(optional, into)]
    disabled: MaybeProp<bool>,
    /// Form field name.
    #[prop(optional, into)]
    name: Option<String>,
) -> impl IntoView {
    let id = crate::id::next_id(SCOPE);
    // Uncontrolled value lives here so the readout and fill can follow the thumb.
    let local = RwSignal::new(default_value.unwrap_or(min));
    let current = move || value.get().unwrap_or_else(|| local.get());
    let text = move || format.map_or_else(|| current().to_string(), |f| f.run(current()));

    view! {
        <div
            data-scope=SCOPE
            data-part="root"
            data-disabled=move || disabled.get().unwrap_or(false).then_some("")
        >
            <div data-scope=SCOPE data-part="header">
                <label data-scope=SCOPE data-part="label" for=id.clone()>{label}</label>
                <span data-scope=SCOPE data-part="value" aria-hidden="true">{text}</span>
            </div>
            <input
                data-scope=SCOPE
                data-part="control"
                type="range"
                id=id
                min=min
                max=max
                step=step
                name=name
                disabled=move || disabled.get().unwrap_or(false)
                aria-valuetext=move || format.map(|f| f.run(current()))
                prop:value=move || current().to_string()
                style=move || format!("--ox-slider-fill: {}%;", fill_percent(current(), min, max))
                on:input=move |ev| {
                    let v = event_target_value(&ev).parse().unwrap_or(min);
                    local.set(v);
                    if let Some(cb) = on_input {
                        cb.run(v);
                    }
                }
            />
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::fill_percent;

    #[test]
    fn fill_is_clamped_and_safe_for_empty_ranges() {
        assert!((fill_percent(25.0, 0.0, 100.0) - 25.0).abs() < f64::EPSILON);
        assert!((fill_percent(150.0, 0.0, 100.0) - 100.0).abs() < f64::EPSILON);
        assert!(fill_percent(5.0, 10.0, 10.0).abs() < f64::EPSILON);
    }
}
