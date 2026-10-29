//! Run the small JS snippets that headless connectors share with the Leptos
//! adapter (`MEASURE_JS` and friends) through `document::eval`.

use std::cell::RefCell;
use std::fmt::Write as _;
use std::rc::Rc;

use dioxus::prelude::*;

/// `s` as a JS string literal. Rust's `{:?}` isn't one: `"\0"` then a digit reads as octal.
pub(crate) fn js_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            // Other controls, plus U+2028/9, which end lines in older engines.
            c if c.is_control() || matches!(c, '\u{2028}' | '\u{2029}') => {
                let _ = write!(out, "\\u{:04x}", u32::from(c));
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// A script that calls `body` with named string arguments and returns its result.
pub(crate) fn call<S: AsRef<str>>(params: &[(&str, S)], body: &str) -> String {
    let names: Vec<&str> = params.iter().map(|(n, _)| *n).collect();
    let values: Vec<String> = params.iter().map(|(_, v)| js_string(v.as_ref())).collect();
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
         const stop = (function(root, send) {{ {body} }})({root}, (v) => dioxus.send(v));\n\
         await dioxus.recv();\n\
         if (typeof stop === \"function\") stop();",
        root = js_string(root)
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

#[cfg(test)]
mod tests {
    use super::js_string;

    #[test]
    fn js_string_escapes() {
        assert_eq!(js_string(r#"a"b\c"#), r#""a\"b\\c""#);
        assert_eq!(js_string("\u{0}1"), r#""\u00001""#);
        assert_eq!(js_string("a\nb\r\t"), r#""a\nb\r\t""#);
        assert_eq!(js_string("x\u{2028}y\u{2029}"), r#""x\u2028y\u2029""#);
        assert_eq!(js_string("\u{1b}"), r#""\u001b""#);
        assert_eq!(js_string("hi 😀 é"), "\"hi 😀 é\"");
    }
}
