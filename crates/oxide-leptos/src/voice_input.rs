//! `<VoiceInput>`: a microphone button that records audio and reports a level meter.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use leptos::prelude::*;
use leptos::wasm_bindgen::closure::Closure;
use leptos::wasm_bindgen::{JsCast, JsValue};
use oxide_headless::voice_input::{
    self as h, VoiceInputCommand, VoiceInputEvent, VoiceInputInput, VoiceInputMachine, VoiceInputPhase, VoiceInputState,
};
use oxide_statechart::Machine;
use web_sys::js_sys;
use web_sys::{AnalyserNode, AudioContext, Blob, BlobEvent, BlobPropertyBag, MediaRecorder, MediaStream, MediaStreamConstraints, MediaStreamTrack};

type OnData = Closure<dyn FnMut(BlobEvent)>;
type OnStop = Closure<dyn FnMut()>;
type Frame = Rc<RefCell<Option<Closure<dyn FnMut()>>>>;

/// What the browser holds while a recording is open. Replaced by the next one.
struct Session {
    recorder: MediaRecorder,
    _handlers: (OnData, OnStop),
}

type Store = StoredValue<Option<Session>, LocalStorage>;

#[derive(Clone, Copy)]
struct Ctx {
    input: Memo<VoiceInputInput>,
    state: RwSignal<VoiceInputState>,
    store: Store,
    on_recorded: Option<Callback<Blob>>,
    on_phase_change: Option<Callback<VoiceInputPhase>>,
}

/// Run one event through the machine, then its commands.
fn send(ctx: Ctx, event: VoiceInputEvent) {
    // Events can still arrive after unmount (the recorder's stop handler).
    let Some(step) = ctx
        .input
        .try_with_untracked(|i| ctx.state.try_with_untracked(|s| VoiceInputMachine::transition(s, event, i)))
        .flatten()
    else {
        return;
    };
    let before = ctx.state.try_get_untracked().map(|s| s.phase);
    let after = step.state.as_ref().map(|s| s.phase);
    if let Some(next) = step.state {
        ctx.state.set(next);
    }
    if let Some(after) = after
        && before != Some(after)
        && let Some(cb) = ctx.on_phase_change
    {
        cb.run(after);
    }
    for command in step.commands {
        match command {
            VoiceInputCommand::Start => start(ctx),
            VoiceInputCommand::Stop => stop(ctx.store),
        }
    }
}

/// Stop the open recording, if any. The recorder's stop handler delivers the audio.
fn stop(store: Store) {
    let _ = store.try_with_value(|s| {
        if let Some(Session { recorder, .. }) = s
            && recorder.state() == web_sys::RecordingState::Recording
        {
            let _ = recorder.stop();
        }
    });
}

/// A microphone permission error, as opposed to a missing device or a crash.
fn is_denied(error: &JsValue) -> bool {
    error
        .dyn_ref::<web_sys::DomException>()
        .is_some_and(|e| matches!(e.name().as_str(), "NotAllowedError" | "PermissionDeniedError" | "SecurityError"))
}

/// Ask for the microphone, record, and feed the level meter. Every outcome comes back through `send`.
fn start(ctx: Ctx) {
    let fail = move |event| send(ctx, event);
    let Some(window) = web_sys::window() else { return fail(VoiceInputEvent::Failed) };
    let Ok(devices) = window.navigator().media_devices() else { return fail(VoiceInputEvent::Failed) };
    let constraints = MediaStreamConstraints::new();
    constraints.set_audio(&JsValue::TRUE);
    let Ok(promise) = devices.get_user_media_with_constraints(&constraints) else {
        return fail(VoiceInputEvent::Failed);
    };
    let ok = Closure::once(move |stream: JsValue| {
        if attach(stream.unchecked_into(), ctx).is_err() {
            send(ctx, VoiceInputEvent::Failed);
        }
    });
    let err = Closure::once(move |error: JsValue| {
        send(ctx, if is_denied(&error) { VoiceInputEvent::Denied } else { VoiceInputEvent::Failed });
    });
    let _ = promise.then2(&ok, &err);
    ok.forget();
    err.forget();
}

fn attach(stream: MediaStream, ctx: Ctx) -> Result<(), JsValue> {
    let recorder = MediaRecorder::new_with_media_stream(&stream)?;
    let chunks: Rc<RefCell<Vec<Blob>>> = Rc::default();
    let running = Rc::new(Cell::new(true));
    let audio = AudioContext::new()?;
    let analyser = audio.create_analyser()?;
    analyser.set_fft_size(256);
    audio.create_media_stream_source(&stream)?.connect_with_audio_node(&analyser)?;

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
            let _ = audio.close();
            let parts = js_sys::Array::new();
            for blob in chunks.borrow().iter() {
                parts.push(blob);
            }
            let options = BlobPropertyBag::new();
            options.set_type(&recorder.mime_type());
            if let (Ok(blob), Some(cb)) = (Blob::new_with_blob_sequence_and_options(&parts, &options), ctx.on_recorded) {
                cb.run(blob);
            }
            send(ctx, VoiceInputEvent::Stopped);
        }
    });
    recorder.set_onstop(Some(on_stop.as_ref().unchecked_ref()));
    recorder.start()?;
    ctx.store.set_value(Some(Session { recorder, _handlers: (on_data, on_stop) }));
    send(ctx, VoiceInputEvent::Granted);
    meter(analyser, running, move |event| send(ctx, event));
    Ok(())
}

/// Report the input level and elapsed time every frame until `running` clears.
fn meter(analyser: AnalyserNode, running: Rc<Cell<bool>>, send: impl Fn(VoiceInputEvent) + 'static) {
    let Some(window) = web_sys::window() else { return };
    let started = window.performance().map_or(0.0, |p| p.now());
    let mut samples = vec![0u8; analyser.fft_size() as usize];
    let frame: Frame = Rc::default();
    let again = frame.clone();
    *again.borrow_mut() = Some(Closure::new(move || {
        let Some(window) = web_sys::window() else { return };
        if !running.get() {
            frame.borrow_mut().take();
            return;
        }
        analyser.get_byte_time_domain_data(&mut samples);
        let power = samples.iter().map(|&b| (f32::from(b) - 128.0) / 128.0).map(|x| x * x).sum::<f32>();
        #[allow(clippy::cast_precision_loss, reason = "a few hundred samples")]
        let rms = (power / samples.len() as f32).sqrt();
        send(VoiceInputEvent::Level((rms * 4.0).min(1.0)));
        let now = window.performance().map_or(started, |p| p.now());
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss, reason = "elapsed time is small and positive")]
        send(VoiceInputEvent::Tick((now - started).max(0.0) as u64));
        if let Some(f) = frame.borrow().as_ref() {
            let _ = window.request_animation_frame(f.as_ref().unchecked_ref());
        }
    }));
    if let Some(f) = again.borrow().as_ref() {
        let _ = window.request_animation_frame(f.as_ref().unchecked_ref());
    }
}

/// A button that records from the microphone, with a level meter. Press to
/// start, press again to stop. The recording arrives in `on_recorded`.
#[component]
pub fn VoiceInput(
    /// Stop on its own after this many seconds.
    #[prop(optional)]
    max_seconds: Option<u32>,
    /// Block recording.
    #[prop(optional, into)]
    disabled: Signal<bool>,
    /// Called with the audio when a recording ends. The `Blob` holds the browser's
    /// native format, usually `audio/webm` or `audio/mp4`.
    #[prop(optional, into)]
    on_recorded: Option<Callback<Blob>>,
    /// Called whenever the recorder phase changes.
    #[prop(optional, into)]
    on_phase_change: Option<Callback<VoiceInputPhase>>,
) -> impl IntoView {
    let input = Memo::new(move |_| VoiceInputInput { max_ms: max_seconds.map(|s| u64::from(s) * 1000), disabled: disabled.get() });
    let state = RwSignal::new(input.with_untracked(VoiceInputMachine::initial));
    let ctx = Ctx { input, state, store: StoredValue::new_local(None), on_recorded, on_phase_change };
    on_cleanup(move || stop(ctx.store));
    let press = move |_| send(ctx, VoiceInputEvent::Press);

    let current = move || state.with(|s: &VoiceInputState| *s);
    view! {
        <div data-scope=h::SCOPE data-part="root" data-state=move || current().phase.as_str()>
            <button
                type="button"
                data-scope=h::SCOPE
                data-part="button"
                aria-pressed=move || h::pressed(current().phase).to_string()
                aria-label=move || h::button_label(current().phase)
                disabled=move || disabled.get()
                // Not `disabled` while asking: that would drop keyboard focus from the button.
                aria-disabled=move || (current().phase == VoiceInputPhase::Requesting).then_some("true")
                on:click=press
            >
                <svg
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="1.75"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    aria-hidden="true"
                    width="22"
                    height="22"
                    inner_html=move || if current().phase == VoiceInputPhase::Recording { h::STOP } else { h::MIC }
                ></svg>
            </button>
            <span data-scope=h::SCOPE data-part="meter" aria-hidden="true">
                {(0..h::BARS)
                    .map(|i| {
                        view! {
                            <span
                                data-scope=h::SCOPE
                                data-part="bar"
                                style=move || format!("transform: scaleY({:.2})", h::bar_scale(i, current().level))
                            ></span>
                        }
                    })
                    .collect_view()}
            </span>
            <span data-scope=h::SCOPE data-part="status" role="status">
                {move || {
                    let s = current();
                    h::status_text(s.phase, s.elapsed_ms)
                }}
            </span>
        </div>
    }
}
