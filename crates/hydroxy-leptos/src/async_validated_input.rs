//! `<AsyncValidatedInput>`: an `Input` that runs the owner's async validator
//! after the user stops typing. Shows idle, validating, valid and invalid
//! states through the input's hint and error. Answers to stale text are dropped.

use std::cell::RefCell;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Poll, Waker};
use std::time::Duration;

use hydroxy_headless::async_validated_input::{
    self as h, AsyncValidatedCommand, AsyncValidatedEvent, AsyncValidatedInput as Props, AsyncValidatedMachine,
    AsyncValidatedState, Messages,
};
use hydroxy_statechart::Machine;
use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::drive::drive;
use crate::input::Input;

pub use hydroxy_headless::async_validated_input::Validity;

/// The owner's validator: text in, `Ok` to accept or `Err(message)` to reject.
pub type Validator = Rc<dyn Fn(String) -> Pin<Box<dyn Future<Output = Result<(), String>>>>>;

/// Resolves after `ms` milliseconds. Handy for faking network latency in demos.
pub async fn sleep(ms: u32) {
    let shared = Rc::new(RefCell::new((false, None::<Waker>)));
    let timer = Rc::clone(&shared);
    set_timeout(
        move || {
            let mut t = timer.borrow_mut();
            t.0 = true;
            if let Some(waker) = t.1.take() {
                waker.wake();
            }
        },
        Duration::from_millis(ms.into()),
    );
    std::future::poll_fn(move |cx| {
        let mut t = shared.borrow_mut();
        if t.0 {
            Poll::Ready(())
        } else {
            t.1 = Some(cx.waker().clone());
            Poll::Pending
        }
    })
    .await;
}

/// Wrap an async closure as a [`Validator`].
pub fn validator<F, Fut>(f: F) -> Validator
where
    F: Fn(String) -> Fut + 'static,
    Fut: Future<Output = Result<(), String>> + 'static,
{
    Rc::new(move |text| Box::pin(f(text)))
}

#[derive(Clone, Copy)]
struct Ctx {
    state: RwSignal<AsyncValidatedState>,
    input: Memo<Props>,
    validator: StoredValue<Validator, LocalStorage>,
    on_change: Option<Callback<String>>,
}

impl Ctx {
    fn send(self, event: AsyncValidatedEvent) {
        for command in drive::<AsyncValidatedMachine, _>(self.state, self.input, event) {
            match command {
                AsyncValidatedCommand::Change(text) => {
                    if let Some(cb) = self.on_change {
                        cb.run(text);
                    }
                }
                // Stale waits are ignored by the machine, so there is nothing to cancel.
                AsyncValidatedCommand::Wait { request, ms } => {
                    set_timeout(
                        move || self.send(AsyncValidatedEvent::Elapsed(request)),
                        Duration::from_millis(ms.into()),
                    );
                }
                AsyncValidatedCommand::Validate { request, value } => {
                    let Some(run) = self.validator.try_with_value(Rc::clone) else { continue };
                    spawn_local(async move {
                        let result = run(value).await;
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
    #[prop(into)]
    label: String,
    /// Runs after the user pauses. Return `Ok(())` to accept or `Err(message)` to reject.
    validator: Validator,
    /// Controlled value. Pair with `on_change`.
    #[prop(optional, into)]
    value: MaybeProp<String>,
    /// Initial value when uncontrolled.
    #[prop(optional, into)]
    default_value: String,
    /// Called with the new text on every keystroke.
    #[prop(optional, into)]
    on_change: Option<Callback<String>>,
    /// Quiet time after the last keystroke before validating. Default 400.
    #[prop(default = 400)]
    debounce_ms: u32,
    /// Help text while there is nothing to report.
    #[prop(optional, into)]
    hint: Option<String>,
    /// Shown while the validator runs. Default "Checking…".
    #[prop(optional, into)]
    validating_text: Option<String>,
    /// Shown when the validator accepts. Default "Looks good.".
    #[prop(optional, into)]
    valid_text: Option<String>,
    /// Placeholder text.
    #[prop(optional, into)]
    placeholder: Option<String>,
    /// Form field name.
    #[prop(optional, into)]
    name: Option<String>,
    /// Block editing.
    #[prop(optional, into)]
    disabled: MaybeProp<bool>,
) -> impl IntoView {
    let input = Memo::new(move |_| Props { value: value.get(), default_value: default_value.clone(), debounce_ms });
    let state = RwSignal::new(input.with_untracked(AsyncValidatedMachine::initial));
    Effect::new(move |_| {
        let synced = input.with(|i| state.with_untracked(|s| AsyncValidatedMachine::sync(s, i)));
        if let Some(synced) = synced {
            state.set(synced);
        }
    });
    let ctx = Ctx { state, input, validator: StoredValue::new_local(validator), on_change };
    let defaults = Messages::default();
    let messages = Messages {
        validating: validating_text.unwrap_or(defaults.validating),
        valid: valid_text.unwrap_or(defaults.valid),
    };
    let messages = StoredValue::new(messages);
    let feedback =
        Memo::new(move |_| state.with(|s| messages.with_value(|m| h::feedback(&s.validity, m, hint.as_deref()))));

    view! {
        <div
            data-scope=h::SCOPE
            data-part="root"
            data-status=move || state.with(|s| h::data_status(&s.validity))
            aria-busy=move || state.with(|s| (s.validity == Validity::Validating).then_some("true"))
        >
            <Input
                label=label
                hint=Signal::derive(move || feedback.get().0)
                error=Signal::derive(move || feedback.get().1)
                value=Signal::derive(move || Some(state.with(|s| s.value.value.clone())))
                on_input=move |text: String| ctx.send(AsyncValidatedEvent::Input(text))
                placeholder=placeholder.unwrap_or_default()
                name=name.unwrap_or_default()
                disabled=disabled
            />
            <div data-scope=h::SCOPE data-part="live" role="status" aria-live="polite" aria-atomic="true">
                {move || state.with(|s| messages.with_value(|m| h::live_text(&s.validity, m)))}
            </div>
        </div>
    }
}
