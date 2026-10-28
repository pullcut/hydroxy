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
