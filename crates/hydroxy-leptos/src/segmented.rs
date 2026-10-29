//! `<Segmented>` / `<SegmentedItem>`: a pill row of mutually exclusive options.
//! Same behaviour as `RadioGroup` (its machine and connector): roving focus,
//! arrow keys that select, controlled or uncontrolled, form participation.
//! Only the look differs.
//!
//! Parts panic if rendered outside their root `Segmented`.

use crate::icon::{Icon, IconName};
use crate::radio_group::Ctx;
use leptos::prelude::*;

pub use hydroxy_headless::radio_group::RadioItem as SegmentedOption;

/// The `data-scope` value on every segmented part.
pub const SCOPE: &str = "segmented";

/// Item padding and type size.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum SegmentedSize {
    /// Compact, for toolbars.
    Sm,
    /// Default.
    #[default]
    Md,
}

impl SegmentedSize {
    /// The `data-size` value.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            SegmentedSize::Sm => "sm",
            SegmentedSize::Md => "md",
        }
    }
}

/// Root. Owns the machine and provides it to the items below.
#[component]
pub fn Segmented(
    /// Accessible name for the group (`aria-label`), such as "Summary length".
    #[prop(into)]
    label: String,
    /// The options, in display order.
    #[prop(into)]
    items: Signal<Vec<SegmentedOption>>,
    /// Controlled value. Pair with `on_value_change`.
    #[prop(optional, into)]
    value: MaybeProp<String>,
    /// Initial value when uncontrolled.
    #[prop(optional, into)]
    default_value: Option<String>,
    /// Disable the whole control.
    #[prop(optional, into)]
    disabled: MaybeProp<bool>,
    /// Item padding and type size.
    #[prop(optional)]
    size: SegmentedSize,
    /// Form field name. When set, the selected value is submitted with the form.
    #[prop(optional, into)]
    name: Option<String>,
    /// Called with the new value whenever the selection changes or is requested.
    #[prop(optional, into)]
    on_value_change: Option<Callback<String>>,
    children: Children,
) -> impl IntoView {
    let ctx = Ctx::provide(SCOPE, items, value, default_value, disabled, true, on_value_change);
    let hidden = ctx.hidden_field(name);

    view! {
        <div
            role="radiogroup"
            aria-label=label
            aria-orientation="horizontal"
            aria-disabled=move || ctx.disabled().then_some("true")
            data-scope=SCOPE
            data-part="root"
            data-size=size.as_str()
        >
            {children()}
            {hidden}
        </div>
    }
}

/// One segment: a label and an optional leading icon.
#[component]
pub fn SegmentedItem(
    /// The option this segment selects.
    #[prop(into)]
    value: String,
    /// Visible label.
    #[prop(into)]
    label: String,
    /// Icon before the label.
    #[prop(optional)]
    icon: Option<IconName>,
) -> impl IntoView {
    let ctx = use_context::<Ctx>().expect("SegmentedItem must be rendered inside <Segmented>");
    let attrs = ctx.item_attrs(&value);
    let (on_click, on_focus) = (value.clone(), value);
    view! {
        <button
            type="button"
            id=move || attrs.with(|x| x.id.clone())
            role="radio"
            aria-checked=move || attrs.with(|x| x.aria_checked).to_string()
            tabindex=move || attrs.with(|x| x.tabindex)
            disabled=move || attrs.with(|x| x.disabled)
            data-scope=SCOPE
            data-part="item"
            data-state=move || attrs.with(|x| x.data_state)
            on:click=move |_| ctx.click(&on_click)
            on:focus=move |_| ctx.focused(&on_focus)
            on:keydown=move |ev| ctx.keydown(&ev)
        >
            {icon.map(|name| view! { <Icon name size=16 /> })}
            {label}
        </button>
    }
}
