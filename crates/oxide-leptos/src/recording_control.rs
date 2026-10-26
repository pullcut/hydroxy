//! `<RecordingControl>`: record, pause, resume and stop from the microphone,
//! with an elapsed timer and a playable result.

use std::cell::RefCell;
use std::rc::Rc;

use leptos::prelude::*;
use leptos::wasm_bindgen::closure::Closure;
use leptos::wasm_bindgen::{JsCast, JsValue};
use oxide_headless::recording_control::{
    self as h, RecordingControlCommand, RecordingControlEvent, RecordingControlInput, RecordingControlMachine,
    RecordingControlState, RecordingError, RecordingPhase,
};
use oxide_statechart::Machine;
use web_sys::js_sys;
use web_sys::{Blob, BlobEvent, BlobPropertyBag, MediaRecorder, MediaStream, MediaStreamConstraints, MediaStreamTrack};

pub use h::RecordingPhase as Phase;

type OnData = Closure<dyn FnMut(BlobEvent)>;
type OnStop = Closure<dyn FnMut()>;
type Frame = Rc<RefCell<Option<Closure<dyn FnMut()>>>>;

/// What the browser holds while a recording is open.
struct Session {
    recorder: MediaRecorder,
    _handlers: (OnData, OnStop),
}

type Store = StoredValue<Option<Session>, LocalStorage>;

/// A finished recording, as shown in the result.
#[derive(Clone, PartialEq)]
struct Finished {
    url: String,
    size: u64,
    name: String,
}

#[derive(Clone, Copy)]
struct Ctx {
    input: Memo<RecordingControlInput>,
    state: RwSignal<RecordingControlState>,
    store: Store,
    result: RwSignal<Option<Finished>>,
    on_recorded: Option<Callback<Blob>>,
    on_phase_change: Option<Callback<RecordingPhase>>,
    serial: StoredValue<u32>,
}

fn send(ctx: Ctx, event: RecordingControlEvent) {
    // Events can still arrive after unmount (the recorder's stop handler).
    let Some(step) = ctx
        .input
        .try_with_untracked(|i| ctx.state.try_with_untracked(|s| RecordingControlMachine::transition(s, event, i)))
        .flatten()
    else {
        return;
    };
    let before = ctx.state.try_get_untracked().map(|s| s.phase);
    let after = step.state.phase;
    ctx.state.set(step.state);
    if before != Some(after)
        && let Some(cb) = ctx.on_phase_change
    {
        cb.run(after);
    }
    for command in step.commands {
        match command {
            RecordingControlCommand::Start => start(ctx),
            RecordingControlCommand::Pause => with_recorder(ctx.store, |r| {
                let _ = r.pause();
            }),
            RecordingControlCommand::Resume => with_recorder(ctx.store, |r| {
                let _ = r.resume();
            }),
            RecordingControlCommand::Stop => stop(ctx.store),
            RecordingControlCommand::Release => release(ctx),
        }
    }
}

fn with_recorder(store: Store, f: impl FnOnce(&MediaRecorder)) {
    let _ = store.try_with_value(|s| {
        if let Some(Session { recorder, .. }) = s {
            f(recorder);
        }
    });
}

/// Stop the open recording, if any. The recorder's stop handler delivers the audio.
fn stop(store: Store) {
    with_recorder(store, |r| {
        if r.state() != web_sys::RecordingState::Inactive {
            let _ = r.stop();
        }
    });
}

/// Free the previous result's object URL.
fn release(ctx: Ctx) {
    if let Some(old) = ctx.result.try_get_untracked().flatten() {
        let _ = web_sys::Url::revoke_object_url(&old.url);
    }
    ctx.result.try_set(None);
}

fn classify(error: &JsValue) -> RecordingError {
    match error.dyn_ref::<web_sys::DomException>().map(web_sys::DomException::name).as_deref() {
        Some("NotAllowedError" | "PermissionDeniedError" | "SecurityError") => RecordingError::Denied,
        Some("NotFoundError" | "OverconstrainedError") => RecordingError::NoDevice,
        _ => RecordingError::Failed,
    }
}

/// Ask for the microphone and record. Every outcome comes back through `send`.
fn start(ctx: Ctx) {
    let fail = move |e| send(ctx, RecordingControlEvent::Failed(e));
    let Some(window) = web_sys::window() else { return fail(RecordingError::Unsupported) };
    let Ok(devices) = window.navigator().media_devices() else { return fail(RecordingError::Unsupported) };
    let constraints = MediaStreamConstraints::new();
    constraints.set_audio(&JsValue::TRUE);
    let Ok(promise) = devices.get_user_media_with_constraints(&constraints) else {
        return fail(RecordingError::Unsupported);
    };
    let ok = Closure::once(move |stream: JsValue| {
        if attach(stream.unchecked_into(), ctx).is_err() {
            fail(RecordingError::Unsupported);
        }
    });
    let err = Closure::once(move |error: JsValue| fail(classify(&error)));
    let _ = promise.then2(&ok, &err);
    ok.forget();
    err.forget();
}

fn attach(stream: MediaStream, ctx: Ctx) -> Result<(), JsValue> {
    let recorder = MediaRecorder::new_with_media_stream(&stream)?;
    let chunks: Rc<RefCell<Vec<Blob>>> = Rc::default();
    let running = Rc::new(std::cell::Cell::new(true));

    let on_data = Closure::<dyn FnMut(BlobEvent)>::new({
        let chunks = chunks.clone();
        move |event: BlobEvent| {
            if let Some(blob) = event.data().filter(|b| b.size() > 0.0) {
                chunks.borrow_mut().push(blob);
            }
        }
    });
    recorder.set_ondataavailable(Some(on_data.as_ref().unchecked_ref()));

    let on_stop = Closure::<dyn FnMut()>::new({
        let (recorder, running) = (recorder.clone(), running.clone());
        move || {
            running.set(false);
            for track in stream.get_tracks().iter() {
                track.unchecked_into::<MediaStreamTrack>().stop();
            }
            let parts = js_sys::Array::new();
            for blob in chunks.borrow().iter() {
                parts.push(blob);
            }
            let options = BlobPropertyBag::new();
            options.set_type(&recorder.mime_type());
            if let Ok(blob) = Blob::new_with_blob_sequence_and_options(&parts, &options) {
                if let Ok(url) = web_sys::Url::create_object_url_with_blob(&blob) {
                    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss, reason = "blob sizes are positive and small")]
                    let size = blob.size() as u64;
                    let n = ctx.serial.try_update_value(|n| {
                        *n += 1;
                        *n
                    });
                    let name = h::file_name(&n.unwrap_or(1).to_string(), &blob.type_());
                    ctx.result.try_set(Some(Finished { url, size, name }));
                }
                if let Some(cb) = ctx.on_recorded {
                    cb.run(blob);
                }
            }
            send(ctx, RecordingControlEvent::Stopped);
        }
    });
    recorder.set_onstop(Some(on_stop.as_ref().unchecked_ref()));
    recorder.start()?;
    ctx.store.set_value(Some(Session { recorder, _handlers: (on_data, on_stop) }));
    send(ctx, RecordingControlEvent::Granted);
    clock(running, move |now| send(ctx, RecordingControlEvent::Tick(now)));
    Ok(())
}

/// Report a monotonic clock reading every frame until `running` clears.
fn clock(running: Rc<std::cell::Cell<bool>>, send: impl Fn(u64) + 'static) {
    let Some(window) = web_sys::window() else { return };
    let frame: Frame = Rc::default();
    let again = frame.clone();
    *again.borrow_mut() = Some(Closure::new(move || {
        let Some(window) = web_sys::window() else { return };
        if !running.get() {
            frame.borrow_mut().take();
            return;
        }
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss, reason = "a clock reading is positive and small")]
        send(window.performance().map_or(0.0, |p| p.now()) as u64);
        if let Some(f) = frame.borrow().as_ref() {
            let _ = window.request_animation_frame(f.as_ref().unchecked_ref());
        }
    }));
    if let Some(f) = again.borrow().as_ref() {
        let _ = window.request_animation_frame(f.as_ref().unchecked_ref());
    }
}

/// One icon in a button.
#[component]
fn Glyph(markup: Signal<&'static str>) -> impl IntoView {
    view! {
        <svg
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="1.75"
            stroke-linecap="round"
            stroke-linejoin="round"
            aria-hidden="true"
            width="18"
            height="18"
            inner_html=move || markup.get()
        ></svg>
    }
}

/// Record from the microphone. Start, pause and resume, then stop; the result
/// plays back in place and arrives in `on_recorded` as a `Blob` in the
/// browser's native format.
#[component]
#[allow(clippy::too_many_lines, clippy::needless_pass_by_value, reason = "one view! tree; props are owned by the macro")]
pub fn RecordingControl(
    /// Stop on its own after this many seconds of recording.
    #[prop(optional)]
    max_seconds: Option<u32>,
    /// Hide the pause button.
    #[prop(optional)]
    no_pause: bool,
    /// Block recording.
    #[prop(optional, into)]
    disabled: Signal<bool>,
    /// Accessible name of the group. Default "Recording".
    #[prop(optional, into)]
    label: Option<String>,
    /// Called with the audio when a recording ends.
    #[prop(optional, into)]
    on_recorded: Option<Callback<Blob>>,
    /// Called whenever the recorder phase changes.
    #[prop(optional, into)]
    on_phase_change: Option<Callback<RecordingPhase>>,
) -> impl IntoView {
    let input = Memo::new(move |_| RecordingControlInput { max_ms: max_seconds.map(|s| u64::from(s) * 1000), disabled: disabled.get(), no_pause });
    let state = RwSignal::new(input.with_untracked(RecordingControlMachine::initial));
    let ctx = Ctx {
        input,
        state,
        store: StoredValue::new_local(None),
        result: RwSignal::new(None),
        on_recorded,
        on_phase_change,
        serial: StoredValue::new(0),
    };
    on_cleanup(move || {
        stop(ctx.store);
        release(ctx);
    });
    let label = label.unwrap_or_else(|| h::LABEL.to_string());
    let phase = move || state.with(|s| s.phase);
    let busy = move || matches!(phase(), RecordingPhase::Requesting | RecordingPhase::Stopping);
    let off = move || disabled.get();

    view! {
        <div
            data-scope=h::SCOPE
            data-part="root"
            role="group"
            aria-label=label
            aria-busy=move || busy().then_some("true")
            data-state=move || phase().as_str()
        >
            <div data-scope=h::SCOPE data-part="buttons">
                <button
                    type="button"
                    data-scope=h::SCOPE
                    data-part="start"
                    aria-label=move || h::start_label(phase())
                    disabled=move || off() || !state.with(RecordingControlState::can_start)
                    on:click=move |_| send(ctx, RecordingControlEvent::Start)
                >
                    <Glyph markup=Signal::derive(|| h::MIC) />
                </button>
                <button
                    type="button"
                    data-scope=h::SCOPE
                    data-part="pause"
                    aria-label=move || h::pause_label(phase())
                    hidden=no_pause
                    disabled=move || off() || !state.with(|s| s.can_pause(&input.get()))
                    on:click=move |_| {
                        let event = if phase() == RecordingPhase::Paused {
                            RecordingControlEvent::Resume
                        } else {
                            RecordingControlEvent::Pause
                        };
                        send(ctx, event);
                    }
                >
                    <Glyph markup=Signal::derive(move || if phase() == RecordingPhase::Paused { h::PLAY } else { h::PAUSE }) />
                </button>
                <button
                    type="button"
                    data-scope=h::SCOPE
                    data-part="stop"
                    aria-label=h::STOP_LABEL
                    disabled=move || off() || !state.with(RecordingControlState::can_stop)
                    on:click=move |_| send(ctx, RecordingControlEvent::Stop)
                >
                    <Glyph markup=Signal::derive(|| h::STOP) />
                </button>
            </div>
            <span data-scope=h::SCOPE data-part="elapsed">{move || h::format_elapsed(state.with(|s| s.elapsed_ms))}</span>
            <span data-scope=h::SCOPE data-part="status" role="status" aria-live="polite">
                {move || state.with(h::status_text)}
            </span>
            {move || {
                ctx.result
                    .get()
                    .map(|r| {
                        view! {
                            <div data-scope=h::SCOPE data-part="result">
                                <audio controls=true src=r.url.clone()></audio>
                                <a data-scope=h::SCOPE data-part="download" href=r.url.clone() download=r.name.clone()>
                                    {format!("Download {} ({})", r.name, h::format_size(r.size))}
                                </a>
                                <button
                                    type="button"
                                    data-scope=h::SCOPE
                                    data-part="discard"
                                    on:click=move |_| send(ctx, RecordingControlEvent::Discard)
                                >
                                    "Discard"
                                </button>
                            </div>
                        }
                    })
            }}
        </div>
    }
}
