//! `Segmented` / `SegmentedItem`: a pill row of mutually exclusive options.
//! Same behaviour as `RadioGroup` (its machine and connector): roving focus,
//! arrow keys that select, controlled or uncontrolled, form participation.
//! Only the look differs.
//!
//! Parts panic if rendered outside their root `Segmented`.

use dioxus::prelude::*;
use hydroxy_headless::radio_group as h;

pub use hydroxy_headless::radio_group::RadioItem as SegmentedOption;

use crate::icon::{Icon, IconName};
use crate::radio_group::{Ctx, use_radio};

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
    label: String,
    /// The options, in display order.
    items: ReadSignal<Vec<SegmentedOption>>,
    /// Controlled value. Pair with `on_value_change`.
    #[props(default)]
    value: ReadSignal<Option<String>>,
    /// Initial value when uncontrolled.
    #[props(default)]
    default_value: Option<String>,
    /// Disable the whole control.
    #[props(default)]
    disabled: ReadSignal<bool>,
    /// Item padding and type size.
    #[props(default)]
    size: SegmentedSize,
    /// Form field name. When set, the selected value is submitted with the form.
    #[props(default)]
    name: Option<String>,
    /// Called with the new value whenever the selection changes or is requested.
    on_value_change: Option<EventHandler<String>>,
    children: Element,
) -> Element {
    let ctx = use_radio(SCOPE, items, value, default_value, disabled, true, on_value_change);
    let selected = ctx.selected();
    rsx! {
        div {
            role: "radiogroup",
            "aria-label": label,
            "aria-orientation": "horizontal",
            "aria-disabled": ctx.input.read().disabled.then_some("true"),
            "data-scope": SCOPE,
            "data-part": "root",
            "data-size": size.as_str(),
            {children}
            if let (Some(name), Some(value)) = (name, selected) {
                input { r#type: "hidden", name, value }
            }
        }
    }
}

/// One segment: a label and an optional leading icon.
#[allow(clippy::expect_used, reason = "rendering outside the parent component is a usage error")]
#[component]
pub fn SegmentedItem(
    /// The option this segment selects.
    value: String,
    /// Visible label.
    label: String,
    /// Icon before the label.
    icon: Option<IconName>,
) -> Element {
    let ctx = try_use_context::<Ctx>().expect("SegmentedItem must be rendered inside Segmented");
    let a = h::item_attrs(&ctx.id.read(), &ctx.state.read(), &ctx.input.read(), &value);
    let (mounted, clicked, focused) = (value.clone(), value.clone(), value);
    rsx! {
        button {
            r#type: "button",
            id: a.id,
            role: a.role,
            "aria-checked": a.aria_checked.to_string(),
            tabindex: a.tabindex,
            disabled: a.disabled,
            "data-scope": SCOPE,
            "data-part": "item",
            "data-state": a.data_state,
            onmounted: move |e| ctx.register(mounted.clone(), e.data()),
            onclick: move |_| ctx.click(clicked.clone()),
            onfocus: move |_| ctx.focus(focused.clone()),
            onkeydown: move |e| ctx.keydown(&e),
            if let Some(name) = icon {
                Icon { name, size: 16 }
            }
            {label}
        }
    }
}
