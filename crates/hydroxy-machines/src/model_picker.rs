//! Model picker: choose a model from the host's list and tune how it runs
//! (temperature, a token budget and reasoning effort). Settings are one value the
//! host owns; every change here produces the next settings for it to store.
//!
//! The model list behaves as a radio group (see [`crate::radio_group`]); this
//! module holds the settings rules: clamping and the labels.

/// One model the host offers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelOption {
    /// Stable id sent to the API, such as "claude-opus".
    pub id: String,
    /// Display name.
    pub name: String,
    /// One line on what it's good at.
    pub description: String,
    /// Context window in tokens.
    pub context_window: u32,
    /// Most tokens one reply may use.
    pub max_output: u32,
    /// Short tags, such as "Fast" or "Vision".
    pub tags: Vec<String>,
}

impl ModelOption {
    /// A model from its id, name, description, context window and output limit.
    #[must_use]
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
        context_window: u32,
        max_output: u32,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: description.into(),
            context_window,
            max_output,
            tags: Vec::new(),
        }
    }

    /// The same model with these tags.
    #[must_use]
    pub fn tags(mut self, tags: &[&str]) -> Self {
        self.tags = tags.iter().map(|t| (*t).to_string()).collect();
        self
    }
}

/// How hard the model thinks before answering.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Effort {
    /// Quick answers.
    Low,
    /// The default balance.
    #[default]
    Medium,
    /// Think it through.
    High,
}

impl Effort {
    /// Every level, low to high.
    pub const ALL: [Effort; 3] = [Effort::Low, Effort::Medium, Effort::High];

    /// The value used in the segmented control.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Effort::Low => "low",
            Effort::Medium => "medium",
            Effort::High => "high",
        }
    }

    /// The visible label.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Effort::Low => "Low",
            Effort::Medium => "Medium",
            Effort::High => "High",
        }
    }
}

/// Parses [`Effort::as_str`].
impl std::str::FromStr for Effort {
    type Err = crate::UnknownName;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Effort::ALL.into_iter().find(|e| e.as_str() == value).ok_or(crate::UnknownName)
    }
}

/// Everything the picker sets.
#[derive(Debug, Clone, PartialEq)]
pub struct ModelSettings {
    /// The chosen model's id.
    pub model: String,
    /// Sampling temperature, 0 to 1.
    pub temperature: f64,
    /// Most tokens per reply.
    pub max_tokens: u32,
    /// Reasoning effort.
    pub effort: Effort,
}

impl Default for ModelSettings {
    fn default() -> Self {
        Self { model: String::new(), temperature: 0.7, max_tokens: 4_096, effort: Effort::Medium }
    }
}

/// Lowest token budget the picker allows.
pub const MIN_TOKENS: u32 = 256;

/// Temperature to two decimal places, kept in 0–1.
#[must_use]
pub fn clamp_temperature(t: f64) -> f64 {
    if t.is_nan() { 0.0 } else { (t.clamp(0.0, 1.0) * 100.0).round() / 100.0 }
}

/// A token budget kept within what `model` can produce.
#[must_use]
pub fn clamp_tokens(tokens: u32, model: Option<&ModelOption>) -> u32 {
    let max = model.map_or(u32::MAX, |m| m.max_output.max(MIN_TOKENS));
    tokens.clamp(MIN_TOKENS, max)
}

/// The settings after choosing `id`: the token budget shrinks to fit the new model.
#[must_use]
pub fn choose_model(settings: &ModelSettings, models: &[ModelOption], id: &str) -> ModelSettings {
    let model = models.iter().find(|m| m.id == id);
    ModelSettings { model: id.to_string(), max_tokens: clamp_tokens(settings.max_tokens, model), ..settings.clone() }
}

/// A token count as people read it: "200K", "1M", "8,192".
#[must_use]
pub fn format_tokens(n: u32) -> String {
    if n >= 1_000_000 && n.is_multiple_of(100_000) {
        let tenths = n / 100_000;
        if tenths.is_multiple_of(10) {
            format!("{}M", tenths / 10)
        } else {
            format!("{}.{}M", tenths / 10, tenths % 10)
        }
    } else if n >= 10_000 && n.is_multiple_of(1_000) {
        format!("{}K", n / 1_000)
    } else {
        let digits = n.to_string();
        let mut out = String::with_capacity(digits.len() + digits.len() / 3);
        for (i, c) in digits.chars().enumerate() {
            if i > 0 && (digits.len() - i).is_multiple_of(3) {
                out.push(',');
            }
            out.push(c);
        }
        out
    }
}

/// The trigger's accessible name: "Model: Opus, high effort".
#[must_use]
pub fn trigger_label(settings: &ModelSettings, models: &[ModelOption]) -> String {
    let name = models.iter().find(|m| m.id == settings.model).map_or("none chosen", |m| m.name.as_str());
    format!("Model: {name}, {} effort", settings.effort.label().to_lowercase())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn models() -> Vec<ModelOption> {
        vec![
            ModelOption::new("big", "Big", "Deep work", 200_000, 32_000),
            ModelOption::new("small", "Small", "Quick replies", 200_000, 8_192).tags(&["Fast"]),
        ]
    }

    #[test]
    fn switching_models_shrinks_the_budget() {
        let s = ModelSettings { model: "big".into(), max_tokens: 20_000, ..ModelSettings::default() };
        let next = choose_model(&s, &models(), "small");
        assert_eq!((next.model.as_str(), next.max_tokens), ("small", 8_192));
        assert_eq!(clamp_tokens(10, None), MIN_TOKENS);
    }

    #[test]
    fn temperature_is_kept_in_range() {
        assert!((clamp_temperature(1.7) - 1.0).abs() < f64::EPSILON);
        assert!((clamp_temperature(0.333) - 0.33).abs() < 1e-9);
        assert!(clamp_temperature(f64::NAN).abs() < f64::EPSILON);
    }

    #[test]
    fn token_counts_read_naturally() {
        assert_eq!(format_tokens(200_000), "200K");
        assert_eq!(format_tokens(1_000_000), "1M");
        assert_eq!(format_tokens(1_500_000), "1.5M");
        assert_eq!(format_tokens(8_192), "8,192");
        assert_eq!(format_tokens(512), "512");
    }

    #[test]
    fn labels() {
        let s = ModelSettings { model: "small".into(), effort: Effort::High, ..ModelSettings::default() };
        assert_eq!(trigger_label(&s, &models()), "Model: Small, high effort");
        assert_eq!("low".parse(), Ok(Effort::Low));
        assert!("max".parse::<Effort>().is_err());
    }
}
