//! Model picker connector. Anatomy: root, trigger (a compact button showing the
//! model and effort; opens the panel), panel (a non-modal popover dialog), models
//! (`role="radiogroup"` of model options, behaving as the radio group connector),
//! model (`role="radio"` with name, description, context and tags), name,
//! description, meta, tag, settings (temperature slider, token budget field,
//! effort segmented control). Every part carries `data-scope="model-picker"`
//! and its `data-part`.

pub use hydroxy_machines::model_picker::*;
pub use hydroxy_machines::radio_group::{RadioGroupCommand, RadioGroupEvent, RadioGroupInput, RadioGroupMachine, RadioItem};

/// The `data-scope` value on every part.
pub const SCOPE: &str = "model-picker";

/// The radio group input for a model list.
#[must_use]
pub fn radio_input(models: &[ModelOption], value: &str) -> RadioGroupInput {
    RadioGroupInput {
        items: models.iter().map(|m| RadioItem::new(m.id.clone())).collect(),
        value: Some(value.to_string()),
        default_value: None,
        disabled: false,
        loop_focus: true,
    }
}

/// A model option's id within the instance `root`.
#[must_use]
pub fn model_id(root: &str, model: &str) -> String {
    crate::radio_group::item_id(root, model)
}

/// The context line under a model: "200K context · 32K output".
#[must_use]
pub fn meta(model: &ModelOption) -> String {
    format!("{} context \u{b7} {} output", format_tokens(model.context_window), format_tokens(model.max_output))
}

/// The trigger's visible text: the model name, or "Choose a model".
#[must_use]
pub fn trigger_text(settings: &ModelSettings, models: &[ModelOption]) -> String {
    models.iter().find(|m| m.id == settings.model).map_or_else(|| "Choose a model".to_string(), |m| m.name.clone())
}

/// The slider readout for a temperature: "0.70".
#[must_use]
pub fn temperature_text(t: f64) -> String {
    format!("{t:.2}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn meta_and_trigger() {
        let m = ModelOption::new("a", "Alpha", "d", 200_000, 64_000);
        assert_eq!(meta(&m), "200K context \u{b7} 64K output");
        let s = ModelSettings { model: "a".into(), ..ModelSettings::default() };
        assert_eq!(trigger_text(&s, std::slice::from_ref(&m)), "Alpha");
        assert_eq!(trigger_text(&ModelSettings::default(), &[m]), "Choose a model");
        assert_eq!(temperature_text(0.7), "0.70");
    }

    #[test]
    fn radio_input_lists_models() {
        let m = [ModelOption::new("a", "A", "", 1, 1), ModelOption::new("b", "B", "", 1, 1)];
        let i = radio_input(&m, "b");
        assert_eq!(i.items.len(), 2);
        assert_eq!(i.value.as_deref(), Some("b"));
        assert_eq!(model_id("mp", "b"), "mp-item-b");
    }
}
