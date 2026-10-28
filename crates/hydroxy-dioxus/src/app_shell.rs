//! `AppShell`: header, navigation, main, aside and footer in one responsive
//! page frame. Wide screens put the navigation beside the main region; narrow
//! ones turn it into a native popover drawer opened from the header.

use dioxus::prelude::*;
use hydroxy_headless::app_shell::{self as h, AppShellCommand, AppShellEvent, AppShellInput, AppShellMachine};
use hydroxy_statechart::Machine;

use crate::drive::drive;
use crate::icon::{Icon, IconName};
use crate::overlay::{hide_popover, show_popover};

pub use h::{NavReason, ScrollMode};

/// Reports `true` or `false` when the browser opens or closes the popover (Escape,
/// light dismiss), and closes the drawer when a link inside it is followed.
fn toggle_script(nav_id: &str) -> String {
    format!(
        "const n = document.getElementById({nav_id:?}); \
         if (n) {{ \
           n.addEventListener('toggle', () => dioxus.send(n.matches(':popover-open'))); \
           n.addEventListener('click', e => {{ if (e.target.closest('a') && n.matches(':popover-open')) n.hidePopover(); }}); \
         }}"
    )
}

/// The page frame. Children are the main region.
#[component]
#[allow(clippy::too_many_lines, reason = "one rsx! tree; splitting it only adds prop plumbing")]
pub fn AppShell(
    /// Navigation content, such as nav items. Rendered once, in one landmark.
    navigation: Element,
    /// Header content, after the menu button.
    header: Option<Element>,
    /// A complementary column after the main region.
    aside: Option<Element>,
    /// Footer content.
    footer: Option<Element>,
    /// Controlled drawer state on narrow screens. Pair with `on_navigation_open_change`.
    #[props(default)]
    navigation_open: ReadSignal<Option<bool>>,
    /// Initial drawer state when uncontrolled.
    #[props(default)]
    default_navigation_open: bool,
    /// The drawer wants to open or close, and why.
    on_navigation_open_change: Option<EventHandler<(bool, NavReason)>>,
    /// `Document` (default) scrolls the page; `Main` scrolls only the main region.
    #[props(default)]
    scroll_mode: ScrollMode,
    /// The shell's height in pixels. Required with `ScrollMode::Main`.
    host_block_size: Option<f64>,
    /// Keep the header visible while the page scrolls.
    #[props(default)]
    sticky_header: bool,
    /// The main region's id, the skip link's target. Pair with `skip_label`.
    main_id: Option<String>,
    /// The skip link's text. Pair with `main_id`.
    skip_label: Option<String>,
    /// Accessible name of the navigation landmark. Default "Navigation".
    navigation_label: Option<String>,
    /// Accessible name of the menu button. Default "Open navigation".
    navigation_trigger_label: Option<String>,
    /// Accessible name of the drawer's close button. Default "Close navigation".
    navigation_close_label: Option<String>,
    /// The main content.
    children: Element,
) -> Element {
    let skip = h::check_skip_link(main_id.as_deref(), skip_label.as_deref()).expect("AppShell skip link");
    assert!(
        scroll_mode == ScrollMode::Document || host_block_size.is_some_and(|b| b.is_finite() && b > 0.0),
        "AppShell main scroll mode needs a positive host_block_size"
    );
    let root_id = use_hook(|| hydroxy_headless::next_id(h::SCOPE));
    let nav_id = h::navigation_id(&root_id);
    let main_dom_id = main_id.clone().filter(|_| skip).unwrap_or_else(|| h::default_main_id(&root_id));

    let input = use_memo(move || AppShellInput { open: navigation_open(), default_open: default_navigation_open });
    let state = use_signal(|| AppShellMachine::initial(&input.peek()));
    let popover_id = nav_id.clone();
    let send = Callback::new(move |event: AppShellEvent| {
        let commands = drive::<AppShellMachine>(state, input, event);
        for command in commands {
            match command {
                AppShellCommand::OpenChange(open, reason) => {
                    if let Some(handler) = on_navigation_open_change {
                        handler.call((open, reason));
                    }
                }
                AppShellCommand::SetPopover(open) => {
                    if open {
                        show_popover(&popover_id);
                    } else {
                        hide_popover(&popover_id);
                    }
                }
            }
        }
    });
    let first = use_hook(|| CopyValue::new(true));
    use_effect({
        let nav_id = nav_id.clone();
        let mut first = first;
        move || {
            input.read();
            if first() {
                first.set(false);
                if state.peek().open {
                    show_popover(&nav_id);
                }
            } else {
                send.call(AppShellEvent::Sync);
            }
        }
    });
    let script = toggle_script(&nav_id);
    use_future(move || {
        let script = script.clone();
        async move {
            let mut wiring = document::eval(&script);
            while let Ok(open) = wiring.recv::<bool>().await {
                send.call(AppShellEvent::Toggled(open));
            }
        }
    });

    let nav_label = navigation_label.unwrap_or_else(|| h::NAVIGATION_LABEL.to_string());
    let trigger_label = navigation_trigger_label.unwrap_or_else(|| h::TRIGGER_LABEL.to_string());
    let close_label = navigation_close_label.unwrap_or_else(|| h::CLOSE_LABEL.to_string());
    let open_now = state.read().open;
    rsx! {
        div {
            "data-scope": h::SCOPE,
            "data-part": "root",
            "data-scroll-mode": scroll_mode.as_str(),
            "data-sticky-header": sticky_header.then_some(""),
            style: h::host_style(scroll_mode, host_block_size),
            if skip {
                a { "data-scope": h::SCOPE, "data-part": "skip-link", href: "#{main_dom_id}", {skip_label.clone()} }
            }
            header { "data-scope": h::SCOPE, "data-part": "header",
                button {
                    r#type: "button",
                    "data-scope": h::SCOPE,
                    "data-part": "menu",
                    "aria-label": trigger_label,
                    "aria-controls": nav_id.clone(),
                    "aria-expanded": open_now.to_string(),
                    onclick: move |_| send.call(AppShellEvent::Open),
                    Icon { name: IconName::Menu, size: 20 }
                }
                {header}
            }
            div { "data-scope": h::SCOPE, "data-part": "body",
                nav {
                    id: nav_id.clone(),
                    "popover": "auto",
                    "aria-label": nav_label,
                    "data-scope": h::SCOPE,
                    "data-part": "navigation",
                    button {
                        r#type: "button",
                        "data-scope": h::SCOPE,
                        "data-part": "close",
                        "aria-label": close_label,
                        onclick: move |_| send.call(AppShellEvent::Close),
                        Icon { name: IconName::X, size: 20 }
                    }
                    {navigation}
                }
                main { id: "{main_dom_id}", tabindex: "-1", "data-scope": h::SCOPE, "data-part": "main", {children} }
                if let Some(aside) = aside {
                    aside { "data-scope": h::SCOPE, "data-part": "aside", {aside} }
                }
            }
            if let Some(footer) = footer {
                footer { "data-scope": h::SCOPE, "data-part": "footer", {footer} }
            }
        }
    }
}
