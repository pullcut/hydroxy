//! Run the small JS snippets that headless connectors share with the Leptos
//! adapter (`MEASURE_JS` and friends) through `document::eval`.

use std::cell::RefCell;
use std::rc::Rc;

use dioxus::prelude::*;

/// A script that calls `body` with named string arguments and returns its result.
pub(crate) fn call<S: AsRef<str>>(params: &[(&str, S)], body: &str) -> String {
    let names: Vec<&str> = params.iter().map(|(n, _)| *n).collect();
    let values: Vec<String> = params.iter().map(|(_, v)| format!("{:?}", v.as_ref())).collect();
    format!("return (function({}) {{ {body} }})({});", names.join(", "), values.join(", "))
}

/// Run `body` with these arguments, ignoring the result.
pub(crate) fn run<S: AsRef<str>>(params: &[(&str, S)], body: &str) {
    let _ = document::eval(&call(params, body));
}

/// Write `text` to the clipboard. Best-effort: without clipboard access nothing is copied.
pub(crate) fn copy(text: &str) {
    run(&[("text", text)], "try { navigator.clipboard.writeText(text); } catch (_) {}");
}

/// Whether the visitor asked the system for less motion. Reads `true` when that can't be found out.
pub(crate) async fn prefers_reduced_motion() -> bool {
    let script = call::<&str>(&[], "return matchMedia('(prefers-reduced-motion: reduce)').matches;");
    document::eval(&script).join::<bool>().await.unwrap_or(true)
}

/// Save `body` as a file called `name` with media type `mime`, through a temporary link.
pub(crate) fn download(name: &str, mime: &str, body: &str) {
    let href = hydroxy_headless::whiteboard::data_url(mime, body);
    run(
        &[("href", href.as_str()), ("name", name)],
        "const a = document.createElement('a'); a.href = href; a.download = name; a.click();",
    );
}

/// Resolves after `ms` milliseconds.
pub(crate) async fn sleep(ms: u32) {
    let _ =
        document::eval(&format!("await new Promise((r) => setTimeout(r, {ms})); return true;")).join::<bool>().await;
}

/// A watcher script: `body` (parameters `root`, `send`, returning a cleanup) runs
/// once "start" arrives, and stops at the next message.
fn watch_script(body: &str, root: &str) -> String {
    format!(
        "if ((await dioxus.recv()) !== \"start\") return;\n\
         const stop = (function(root, send) {{ {body} }})({root:?}, (v) => dioxus.send(v));\n\
         await dioxus.recv();\n\
         if (typeof stop === \"function\") stop();"
    )
}

/// Start a watcher on `root` and call `on` with each report. Send it "start"
/// once the element is mounted; it stops when the component drops.
pub(crate) fn use_watch(body: &'static str, root: String, on: impl FnMut(String) + 'static) -> document::Eval {
    let watch = use_hook(move || document::eval(&watch_script(body, &root)));
    let on = use_hook(move || Rc::new(RefCell::new(on)));
    use_future(move || {
        let mut w = watch;
        let on = on.clone();
        async move {
            while let Ok(report) = w.recv::<String>().await {
                (on.borrow_mut())(report);
            }
        }
    });
    use_drop(move || {
        let _ = watch.send("stop");
    });
    watch
}
