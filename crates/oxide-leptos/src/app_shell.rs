//! `<AppShell>`: header, navigation, main, aside and footer in one responsive
//! page frame. Wide screens put the navigation beside the main region; narrow
//! ones turn it into a native popover drawer opened from the header.

use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use oxide_headless::app_shell::{
    self as h, AppShellCommand, AppShellEvent, AppShellInput, AppShellMachine, AppShellState,
};
use oxide_statechart::Machine;

use crate::overlay::{hide_popover, show_popover};

pub use h::{NavReason, ScrollMode};

/// The page frame. Children are the main region.
#[component]
#[allow(clippy::too_many_lines, clippy::needless_pass_by_value, reason = "one view! tree; props are owned by the macro")]
pub fn AppShell(
    /// Navigation content, such as nav items. Rendered once, in one landmark.
    #[prop(into)]
    navigation: ViewFn,
    /// Header content, after the menu button.
    #[prop(optional, into)]
    header: Option<ViewFn>,
    /// A complementary column after the main region.
    #[prop(optional, into)]
    aside: Option<ViewFn>,
    /// Footer content.
    #[prop(optional, into)]
    footer: Option<ViewFn>,
    /// Controlled drawer state on narrow screens. Pair with `on_navigation_open_change`.
    #[prop(optional, into)]
    navigation_open: MaybeProp<bool>,
    /// Initial drawer state when uncontrolled.
    #[prop(optional)]
    default_navigation_open: bool,
    /// The drawer wants to open or close, and why.
    #[prop(optional, into)]
    on_navigation_open_change: Option<Callback<(bool, NavReason)>>,
    /// `Document` (default) scrolls the page; `Main` scrolls only the main region.
    #[prop(optional)]
    scroll_mode: ScrollMode,
    /// The shell's height in pixels. Required with `ScrollMode::Main`.
    #[prop(optional)]
    host_block_size: Option<f64>,
    /// Keep the header visible while the page scrolls.
    #[prop(optional)]
    sticky_header: bool,
    /// The main region's id, the skip link's target. Pair with `skip_label`.
    #[prop(optional, into)]
    main_id: Option<String>,
    /// The skip link's text. Pair with `main_id`.
    #[prop(optional, into)]
    skip_label: Option<String>,
    /// Accessible name of the navigation landmark. Default "Navigation".
    #[prop(optional, into)]
    navigation_label: Option<String>,
    /// Accessible name of the menu button. Default "Open navigation".
    #[prop(optional, into)]
    navigation_trigger_label: Option<String>,
    /// Accessible name of the drawer's close button. Default "Close navigation".
    #[prop(optional, into)]
    navigation_close_label: Option<String>,
    /// The main content.
    children: Children,
) -> impl IntoView {
    let skip = h::check_skip_link(main_id.as_deref(), skip_label.as_deref()).expect("AppShell skip link");
    assert!(
        scroll_mode == ScrollMode::Document || host_block_size.is_some_and(|b| b.is_finite() && b > 0.0),
        "AppShell main scroll mode needs a positive host_block_size"
    );
    let root_id = crate::id::next_id(h::SCOPE);
    let nav_id = h::navigation_id(&root_id);
    let main_dom_id = main_id.clone().filter(|_| skip).unwrap_or_else(|| h::default_main_id(&root_id));
    let nav_label = navigation_label.unwrap_or_else(|| h::NAVIGATION_LABEL.to_string());
    let trigger_label = navigation_trigger_label.unwrap_or_else(|| h::TRIGGER_LABEL.to_string());
    let close_label = navigation_close_label.unwrap_or_else(|| h::CLOSE_LABEL.to_string());
    let nav_id = StoredValue::new(nav_id);

    let input = Memo::new(move |_| AppShellInput { open: navigation_open.get(), default_open: default_navigation_open });
    let state = RwSignal::new(input.with_untracked(AppShellMachine::initial));
    let send = move |event: AppShellEvent| {
        let Some(step) = input
            .try_with_untracked(|i| state.try_with_untracked(|s| AppShellMachine::transition(s, event, i)))
            .flatten()
        else {
            return;
        };
        state.set(step.state);
        for command in step.commands {
            match command {
                AppShellCommand::OpenChange(open, reason) => {
                    if let Some(cb) = on_navigation_open_change {
                        cb.run((open, reason));
                    }
                }
                AppShellCommand::SetPopover(open) => {
                    let id = nav_id.get_value();
                    if open {
                        show_popover(&id);
                    } else {
                        hide_popover(&id);
                    }
                }
            }
        }
    };
    Effect::new(move |previous: Option<()>| {
        input.track();
        if previous.is_none() {
            if state.with_untracked(|s: &AppShellState| s.open) {
                show_popover(&nav_id.get_value());
            }
        } else {
            send(AppShellEvent::Sync);
        }
    });

    // The browser closes the popover on Escape and outside clicks; follow it.
    let toggled = move |ev: web_sys::Event| {
        let open = ev
            .target()
            .and_then(|t| t.dyn_into::<web_sys::Element>().ok())
            .is_some_and(|el| el.matches(":popover-open").unwrap_or(false));
        send(AppShellEvent::Toggled(open));
    };
    // Following a link in the drawer closes it.
    let close_on_link = move |ev: web_sys::MouseEvent| {
        let on_link = ev
            .target()
            .and_then(|t| t.dyn_into::<web_sys::Element>().ok())
            .and_then(|el| el.closest("a").ok().flatten())
            .is_some();
        if on_link {
            send(AppShellEvent::Close);
        }
    };

    view! {
        <div
            data-scope=h::SCOPE
            data-part="root"
            data-scroll-mode=scroll_mode.as_str()
            data-sticky-header=sticky_header.then_some("")
            style=h::host_style(scroll_mode, host_block_size)
        >
            {skip
                .then(|| {
                    view! {
                        <a data-scope=h::SCOPE data-part="skip-link" href=format!("#{main_dom_id}")>
                            {skip_label.clone()}
                        </a>
                    }
                })}
            <header data-scope=h::SCOPE data-part="header">
                <button
                    type="button"
                    data-scope=h::SCOPE
                    data-part="menu"
                    aria-label=trigger_label
                    aria-controls=nav_id.get_value()
                    aria-expanded=move || state.with(|s| s.open).to_string()
                    on:click=move |_| send(AppShellEvent::Open)
                >
                    <Glyph markup=h::MENU />
                </button>
                {header.map(|f| f.run())}
            </header>
            <div data-scope=h::SCOPE data-part="body">
                <nav
                    id=nav_id.get_value()
                    popover="auto"
                    aria-label=nav_label
                    data-scope=h::SCOPE
                    data-part="navigation"
                    on:toggle=toggled
                    on:click=close_on_link
                >
                    <button
                        type="button"
                        data-scope=h::SCOPE
                        data-part="close"
                        aria-label=close_label
                        on:click=move |_| send(AppShellEvent::Close)
                    >
                        <Glyph markup=h::CLOSE />
                    </button>
                    {navigation.run()}
                </nav>
                <main id=main_dom_id tabindex="-1" data-scope=h::SCOPE data-part="main">
                    {children()}
                </main>
                {aside.map(|f| view! { <aside data-scope=h::SCOPE data-part="aside">{f.run()}</aside> })}
            </div>
            {footer.map(|f| view! { <footer data-scope=h::SCOPE data-part="footer">{f.run()}</footer> })}
        </div>
    }
}

/// A 20px line icon from headless markup.
#[component]
fn Glyph(markup: &'static str) -> impl IntoView {
    view! {
        <svg
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="1.75"
            stroke-linecap="round"
            stroke-linejoin="round"
            aria-hidden="true"
            width="20"
            height="20"
            inner_html=markup
        ></svg>
    }
}
