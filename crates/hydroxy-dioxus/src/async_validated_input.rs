//! `AsyncValidatedInput`: an `Input` that runs the owner's async validator
//! after the user stops typing. Shows idle, validating, valid and invalid
//! states through the input's hint and error. Answers to stale text are dropped.

use std::future::Future;
use std::pin::Pin;

use dioxus::prelude::*;
use hydroxy_headless::async_validated_input::{
    self as h, AsyncValidatedCommand, AsyncValidatedEvent, AsyncValidatedInput as Props, AsyncValidatedMachine,
    AsyncValidatedState, Messages,
};
use hydroxy_statechart::Machine;

use crate::input::Input;

pub use hydroxy_headless::async_validated_input::Validity;

/// What a validator returns: `Ok` to accept, `Err(message)` to reject.
pub type ValidateFuture = Pin<Box<dyn Future<Output = Result<(), String>>>>;

/// Resolves after `ms` milliseconds. Handy for faking network latency in demos.
pub async fn sleep(ms: u32) {
    let js = format!("await new Promise(r => setTimeout(r, {ms})); return true;");
    let _ = document::eval(&js).join::<bool>().await;
}

/// Wrap an async closure as a validator prop.
pub fn validator<F, Fut>(f: F) -> Callback<String, ValidateFuture>
where
    F: Fn(String) -> Fut + 'static,
    Fut: Future<Output = Result<(), String>> + 'static,
{
    Callback::new(move |text| -> ValidateFuture { Box::pin(f(text)) })
}

#[derive(Clone, Copy)]
struct Ctx {
    state: Signal<AsyncValidatedState>,
    input: Memo<Props>,
    validator: Callback<String, ValidateFuture>,
    on_change: Option<EventHandler<String>>,
}

impl Ctx {
    fn send(mut self, event: AsyncValidatedEvent) {
        let step = AsyncValidatedMachine::transition(&self.state.peek(), event, &self.input.peek());
        if let Some(next) = step.state {
            self.state.set(next);
        }
        for command in step.commands {
            match command {
                AsyncValidatedCommand::Change(text) => {
                    if let Some(handler) = self.on_change {
                        handler.call(text);
                    }
                }
                // The clock runs through eval, so it works on web and desktop alike.
                // Stale waits are ignored by the machine, so there is nothing to cancel.
                AsyncValidatedCommand::Wait { request, ms } => {
                    spawn(async move {
                        let js = format!("await new Promise(r => setTimeout(r, {ms})); return true;");
                        let _ = document::eval(&js).join::<bool>().await;
                        self.send(AsyncValidatedEvent::Elapsed(request));
                    });
                }
                AsyncValidatedCommand::Validate { request, value } => {
                    let run = self.validator;
                    spawn(async move {
                        let result = run.call(value).await;
                        self.send(AsyncValidatedEvent::Resolved { request, result });
                    });
                }
            }
        }
    }
}

/// A text field checked by an async validator.
#[component]
pub fn AsyncValidatedInput(
    /// Visible label.
    label: String,
    /// Runs after the user pauses. Build one with [`validator`].
    validator: Callback<String, ValidateFuture>,
    /// Controlled value. Pair with `on_change`.
    #[props(default)]
    value: ReadSignal<Option<String>>,
    /// Initial value when uncontrolled.
    #[props(default)]
    default_value: String,
    /// Called with the new text on every keystroke.
    on_change: Option<EventHandler<String>>,
    /// Quiet time after the last keystroke before validating. Default 400.
    #[props(default = 400)]
    debounce_ms: u32,
    /// Help text while there is nothing to report.
    hint: Option<String>,
    /// Shown while the validator runs. Default "Checking…".
    validating_text: Option<String>,
    /// Shown when the validator accepts. Default "Looks good.".
    valid_text: Option<String>,
    /// Placeholder text.
    placeholder: Option<String>,
    /// Form field name.
    name: Option<String>,
    /// Block editing.
    #[props(default)]
    disabled: bool,
) -> Element {
    let input = use_memo(move || Props { value: value(), default_value: default_value.clone(), debounce_ms });
    let mut state = use_signal(|| AsyncValidatedMachine::initial(&input.peek()));
    use_effect(move || {
        let synced = AsyncValidatedMachine::sync(&state.peek(), &input.read());
        if let Some(synced) = synced {
            state.set(synced);
        }
    });
    let ctx = use_hook(|| Ctx { state, input, validator, on_change });
    let defaults = Messages::default();
    let messages = Messages {
        validating: validating_text.unwrap_or(defaults.validating),
        valid: valid_text.unwrap_or(defaults.valid),
    };
    let s = state.read();
    let (hint, error) = h::feedback(&s.validity, &messages, hint.as_deref());

    rsx! {
        div {
            "data-scope": h::SCOPE,
            "data-part": "root",
            "data-status": h::data_status(&s.validity),
            "aria-busy": (s.validity == Validity::Validating).then_some("true"),
            Input {
                label,
                hint,
                error,
                value: Some(s.value.clone()),
                on_input: move |text: String| ctx.send(AsyncValidatedEvent::Input(text)),
                placeholder,
                name,
                disabled,
            }
            div {
                "data-scope": h::SCOPE,
                "data-part": "live",
                role: "status",
                "aria-live": "polite",
                "aria-atomic": "true",
                {h::live_text(&s.validity, &messages)}
            }
        }
    }
}
