//! `<ThemeEditor>`: edit hydroxy's design tokens live. Pick a preset, then change
//! colours (with the colour picker), corner radii, the spacing scale and the
//! fonts. A preview region shows the result, contrast pairs that fall below
//! WCAG AA are flagged, and the theme exports as CSS or JSON.

use hydroxy_headless::theme_editor::{
    self as h, COLOR_TOKENS, FONT_TOKENS, RADIUS_TOKENS, SPACE_SCALE, SPACE_SCALE_RANGE, ThemeEditorCommand,
    ThemeEditorEvent, ThemeEditorMachine, ThemeEditorState,
};
use hydroxy_statechart::Machine;
use leptos::prelude::*;

use crate::badge::{Badge, Tone};
use crate::button::{Button, ButtonVariant};
use crate::callout::{Callout, CalloutKind};
use crate::card::Card;
use crate::code_block::CodeBlock;
use crate::color_picker::ColorPicker;
use crate::icon::{Icon, IconName};
use crate::input::Input;
use crate::popover::{Popover, PopoverContent, PopoverTrigger};
use crate::segmented::{Segmented, SegmentedItem, SegmentedOption};
use crate::select::{Select, SelectOption};
use crate::slider::Slider;

#[derive(Clone, Copy)]
struct Ctx {
    state: RwSignal<ThemeEditorState>,
    input: StoredValue<h::ThemeEditorInput>,
    on_change: Option<Callback<Vec<(String, String)>>>,
}

impl Ctx {
    fn send(self, event: ThemeEditorEvent) {
        let Some(step) =
            self.state.try_with_untracked(|s| self.input.with_value(|i| ThemeEditorMachine::transition(s, event, i)))
        else {
            return;
        };
        if let Some(next) = step.state
            && self.state.with_untracked(|s| *s != next)
        {
            self.state.set(next);
        }
        for ThemeEditorCommand::Change(tokens) in step.commands {
            if let Some(cb) = self.on_change {
                cb.run(tokens);
            }
        }
    }

    fn modified(self, token: &'static str) -> Signal<bool> {
        let state = self.state;
        Signal::derive(move || state.with(|s| s.modified(token)))
    }
}

/// A design-token editor with a scoped live preview.
#[component]
pub fn ThemeEditor(
    /// The preset to start from, as its `data-theme` value. Default `light`.
    #[prop(optional, into)]
    default_preset: Option<String>,
    /// Called with every token (without `--`) and its value after each change.
    #[prop(optional, into)]
    on_change: Option<Callback<Vec<(String, String)>>>,
) -> impl IntoView {
    let input = h::input(hydroxy_tokens::CSS, default_preset.as_deref().unwrap_or("light"));
    let state = RwSignal::new(ThemeEditorState::new(&input));
    let ctx = Ctx { state, input: StoredValue::new(input), on_change };
    let root = crate::id::next_id(h::SCOPE);
    let heading = |part: &str| format!("{root}-{part}");

    let issues = Memo::new(move |_| state.with(ThemeEditorState::contrast_issues));
    let format = RwSignal::new(String::from("css"));
    let preset_options: Vec<SelectOption> = h::presets().into_iter().map(|(v, l)| SelectOption::new(v, l)).collect();

    let color_rows = COLOR_TOKENS
        .iter()
        .map(|&(token, label)| {
            let modified = ctx.modified(token);
            let hex = move || state.with(|s| s.value(token).to_string());
            let issue = move || issues.with(|i| h::token_issue(i, token));
            view! {
                <div data-scope=h::SCOPE data-part="row" data-modified=move || modified.get().then_some("")>
                    <Popover>
                        <PopoverTrigger variant=ButtonVariant::Ghost>
                            <span data-scope=h::SCOPE data-part="swatch" style=move || format!("--swatch: {};", hex())></span>
                            <span data-scope=h::SCOPE data-part="name">{label}</span>
                            <span data-scope=h::SCOPE data-part="value">{hex}</span>
                        </PopoverTrigger>
                        <PopoverContent>
                            <ColorPicker
                                label=label
                                value=Signal::derive(move || Some(state.with(|s| h::picker_value(s, token))))
                                swatches=Vec::new()
                                on_change=move |c| ctx.send(h::color_event(token, c))
                            />
                        </PopoverContent>
                    </Popover>
                    {move || issue().map(|i| view! {
                        <span data-scope=h::SCOPE data-part="warning" title=h::issue_text(&i)>
                            <Icon name=IconName::TriangleAlert size=14 />
                            <span data-scope=h::SCOPE data-part="sr">"Low contrast "</span>
                            {h::ratio_text(i.ratio)}
                        </span>
                    })}
                    <ResetButton ctx=ctx label=label token=token />
                </div>
            }
        })
        .collect_view();

    let radius_rows = RADIUS_TOKENS
        .iter()
        .map(|&(token, label)| {
            let modified = ctx.modified(token);
            view! {
                <div data-scope=h::SCOPE data-part="row" data-modified=move || modified.get().then_some("")>
                    <Slider
                        label=format!("{label} radius")
                        min=0.0
                        max=h::RADIUS_MAX
                        value=Signal::derive(move || Some(state.with(|s| h::radius(s, token))))
                        on_input=move |v| ctx.send(h::radius_event(token, v))
                        format=|v: f64| format!("{v}px")
                    />
                    <ResetButton ctx=ctx label=label token=token />
                </div>
            }
        })
        .collect_view();

    let font_rows = FONT_TOKENS
        .iter()
        .map(|&(token, label)| {
            let modified = ctx.modified(token);
            let options = Signal::derive(move || {
                state.with(|s| {
                    h::font_options(s, token).into_iter().map(|o| SelectOption::new(o.clone(), o)).collect::<Vec<_>>()
                })
            });
            view! {
                <div data-scope=h::SCOPE data-part="row" data-modified=move || modified.get().then_some("")>
                    <Select
                        label=format!("{label} font")
                        options=options
                        value=Signal::derive(move || Some(state.with(|s| h::font_choice(s.value(token)))))
                        on_change=move |choice: String| {
                            let event = state.with_untracked(|s| h::font_event(s, token, &choice));
                            ctx.send(event);
                        }
                    />
                    <ResetButton ctx=ctx label=label token=token />
                </div>
            }
        })
        .collect_view();

    let space_modified = ctx.modified(SPACE_SCALE);

    view! {
        <div data-scope=h::SCOPE data-part="root" role="group" aria-label="Theme editor">
            <div data-scope=h::SCOPE data-part="controls">
                <div data-scope=h::SCOPE data-part="toolbar">
                    <Select
                        label="Preset"
                        options=preset_options
                        value=Signal::derive(move || Some(state.with(|s| s.preset.clone())))
                        on_change=move |p: String| ctx.send(ThemeEditorEvent::SetPreset(p))
                    />
                    <Button
                        variant=ButtonVariant::Secondary
                        disabled=Signal::derive(move || !state.with(ThemeEditorState::is_dirty))
                        on_click=move |_| ctx.send(ThemeEditorEvent::ResetAll)
                    >
                        "Reset all"
                    </Button>
                </div>
                <section data-scope=h::SCOPE data-part="section" aria-labelledby=heading("colours")>
                    <h3 id=heading("colours")>"Colours"</h3>
                    {color_rows}
                </section>
                <section data-scope=h::SCOPE data-part="section" aria-labelledby=heading("corners")>
                    <h3 id=heading("corners")>"Corners"</h3>
                    {radius_rows}
                </section>
                <section data-scope=h::SCOPE data-part="section" aria-labelledby=heading("spacing")>
                    <h3 id=heading("spacing")>"Spacing"</h3>
                    <div data-scope=h::SCOPE data-part="row" data-modified=move || space_modified.get().then_some("")>
                        <Slider
                            label="Spacing scale"
                            min=f64::from(SPACE_SCALE_RANGE.0)
                            max=f64::from(SPACE_SCALE_RANGE.1)
                            step=5.0
                            value=Signal::derive(move || Some(f64::from(state.with(|s| s.space_scale))))
                            on_input=move |v| ctx.send(h::space_event(v))
                            format=|v: f64| format!("{v}%")
                        />
                        <ResetButton ctx=ctx label="Spacing scale" token=SPACE_SCALE />
                    </div>
                </section>
                <section data-scope=h::SCOPE data-part="section" aria-labelledby=heading("type")>
                    <h3 id=heading("type")>"Type"</h3>
                    {font_rows}
                </section>
            </div>
            <div data-scope=h::SCOPE data-part="output">
                <div
                    role="region"
                    aria-label="Preview"
                    data-scope=h::SCOPE
                    data-part="preview"
                    data-theme=move || state.with(|s| s.preset.clone())
                    style=move || state.with(ThemeEditorState::style)
                >
                    <Preview />
                </div>
                <div data-scope=h::SCOPE data-part="contrast" data-ok=move || issues.with(Vec::is_empty).then_some("")>
                    <p data-scope=h::SCOPE data-part="live" role="status" aria-live="polite">
                        {move || issues.with(|i| h::summary(i))}
                    </p>
                    <ul data-scope=h::SCOPE data-part="issues">
                        {move || issues.get().iter().map(|i| view! { <li>{h::issue_text(i)}</li> }).collect_view()}
                    </ul>
                </div>
                <div data-scope=h::SCOPE data-part="export">
                    <Segmented
                        label="Export format"
                        items={h::EXPORT_FORMATS.iter().map(|(v, _)| SegmentedOption::new(*v)).collect::<Vec<_>>()}
                        default_value="css"
                        on_value_change=move |v| format.set(v)
                    >
                        {h::EXPORT_FORMATS.iter().map(|(v, l)| view! { <SegmentedItem value=*v label=*l /> }).collect_view()}
                    </Segmented>
                    {move || {
                        let code = format.with(|f| state.with(|s| h::export(s, f)));
                        let lang = format.get();
                        view! { <CodeBlock code=code lang=lang /> }
                    }}
                </div>
            </div>
        </div>
    }
}

#[component]
fn ResetButton(ctx: Ctx, label: &'static str, token: &'static str) -> impl IntoView {
    let modified = ctx.modified(token);
    view! {
        <button
            type="button"
            data-scope=h::SCOPE
            data-part="reset"
            aria-label=format!("Reset {label}")
            disabled=move || !modified.get()
            on:click=move |_| ctx.send(ThemeEditorEvent::Reset(token.to_string()))
        >
            "Reset"
        </button>
    }
}

/// Realistic UI built from hydroxy components, so every token shows somewhere.
#[component]
fn Preview() -> impl IntoView {
    view! {
        <Card>
            <div data-scope=h::SCOPE data-part="sample">
                <h4>"Invoice INV-2041"</h4>
                <p>"Northwind Traders · due 14 November · " <a href="#preview" on:click=|ev| ev.prevent_default()>"View PDF"</a></p>
                <div data-scope=h::SCOPE data-part="badges">
                    <Badge tone=Tone::Success dot=true>"Paid"</Badge>
                    <Badge tone=Tone::Warning dot=true>"Partial"</Badge>
                    <Badge tone=Tone::Danger dot=true>"Overdue"</Badge>
                    <Badge tone=Tone::Info dot=true>"Draft"</Badge>
                </div>
                <Input label="Reference" default_value="PO-88123" hint="Shown on the customer's statement" />
                <Callout kind=CalloutKind::Warning title="Card expires soon">
                    "The card on file expires at the end of the month."
                </Callout>
                <div data-scope=h::SCOPE data-part="actions">
                    <Button variant=ButtonVariant::Primary>"Send invoice"</Button>
                    <Button variant=ButtonVariant::Accent>"Record payment"</Button>
                    <Button variant=ButtonVariant::Secondary>"Cancel"</Button>
                </div>
                <small>"Amounts in GBP · " <code>"£1,240.00"</code></small>
            </div>
        </Card>
    }
}
