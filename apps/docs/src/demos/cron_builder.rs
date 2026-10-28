use leptos::prelude::*;
use oxide_leptos::cron_builder::CronBuilder;

pub fn demo() -> AnyView {
    let (saved, set_saved) = signal(String::from("0 9 * * 1-5"));

    view! {
        <div style="display: grid; gap: var(--space-3);">
            <CronBuilder label="Digest email schedule" value=saved on_change=move |v| set_saved.set(v) />
            <p style="margin: 0; font: var(--text-xs); color: var(--text-3);">
                "Saved: " <code>{saved}</code> " · try typing " <code>"*/20 8-18 * * MON-FRI"</code>
            </p>
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "cron-builder",
    name: "Cron builder",
    group: "Forms",
    summary: "Build a schedule in plain language or type a cron expression. Each updates the other, the schedule is read back in words, and the next five runs are listed in UTC.",
    demo,
    leptos: include_str!("cron_builder.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/cron_builder.rs"),
    parts: &[
        &["CronBuilder", "root", "role=\"group\" named by label. data-frequency, data-disabled"],
        &["CronBuilder", "frequency", "<select> of presets: minutes, hours, daily, weekdays, weekly, monthly, custom"],
        &["CronBuilder", "interval", "<select> of minute or hour steps that divide evenly"],
        &["CronBuilder", "time / minute", "<input type=\"time\"> in UTC, or minutes past the hour"],
        &["CronBuilder", "weekdays / weekday", "Group of <button aria-pressed> toggles, Monday first. The last day can't be turned off"],
        &["CronBuilder", "month-day", "<select> of 1st to 31st"],
        &["CronBuilder", "expression", "Mono text <input>. aria-invalid and the error when it doesn't parse"],
        &["CronBuilder", "description", "Polite live text: \"At 09:00 on weekdays\""],
        &["CronBuilder", "runs / run", "<ol> of <time datetime> for the next five runs"],
    ],
    keys: &[
        &["Tab", "Moves through the controls, then the expression"],
        &["Arrow keys", "In a select or the time input: change the value"],
        &["Enter / Space", "On a weekday: toggle it"],
    ],
    props: &[
        &["value", "Option<String>", "Controlled expression. Pair with on_change"],
        &["default_value", "String", "Initial expression when uncontrolled. Blank is every day at 09:00"],
        &["on_change", "Callback<String>", "Called with each new valid expression"],
        &["now", "i64", "Unix seconds to list runs after. Default: the browser's clock"],
        &["label", "String", "Accessible name for the group. Default \"Schedule\""],
        &["disabled", "bool", "Block editing"],
    ],
};
