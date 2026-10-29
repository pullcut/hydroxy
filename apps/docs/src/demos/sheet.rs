use hydroxy_leptos::button::ButtonVariant;
use hydroxy_leptos::sheet::*;
use leptos::prelude::*;

pub fn demo() -> AnyView {
    let (filters_open, set_filters_open) = signal(false);

    view! {
        <div style="display: flex; flex-wrap: wrap; gap: var(--space-3);">
            // Uncontrolled, from the inline end edge (right in LTR).
            <Sheet>
                <SheetTrigger variant=ButtonVariant::Primary>"Project details"</SheetTrigger>
                <SheetContent>
                    <SheetHeader>
                        <SheetTitle>"Q3 customer research"</SheetTitle>
                        <SheetDescription>"12 files, 3 summaries. Updated 2 hours ago."</SheetDescription>
                        <SheetCloseIcon />
                    </SheetHeader>
                    <SheetBody>
                        <p>"Interviews with 18 customers across retail and logistics, grouped by theme."</p>
                        <p>"Summaries cite their sources, so every claim links back to a transcript."</p>
                    </SheetBody>
                    <SheetFooter>
                        <SheetClose variant=ButtonVariant::Secondary>"Close"</SheetClose>
                        <SheetClose variant=ButtonVariant::Accent>"Open project"</SheetClose>
                    </SheetFooter>
                </SheetContent>
            </Sheet>

            // Controlled, from the bottom edge.
            <Sheet open=filters_open on_open_change=move |open| set_filters_open.set(open)>
                <SheetTrigger variant=ButtonVariant::Secondary>"Filters"</SheetTrigger>
                <SheetContent side=SheetSide::Bottom>
                    <SheetHeader>
                        <SheetTitle>"Filter sources"</SheetTitle>
                        <SheetCloseIcon />
                    </SheetHeader>
                    <SheetBody>
                        <p>"Show transcripts, notes and summaries from the last 90 days."</p>
                    </SheetBody>
                    <SheetFooter>
                        <SheetClose variant=ButtonVariant::Primary>"Apply filters"</SheetClose>
                    </SheetFooter>
                </SheetContent>
            </Sheet>
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "sheet",
    name: "Sheet",
    group: "Overlays",
    summary: "A panel that slides in from an edge of the screen, for details and settings beside the page. A modal native <dialog>, so focus stays inside and returns to the trigger on close.",
    demo,
    leptos: include_str!("sheet.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/sheet.rs"),
    parts: &[
        &["Sheet", "-", "Provides the machine. Renders no element"],
        &["SheetTrigger", "trigger", "button, aria-haspopup=dialog, aria-expanded, aria-controls"],
        &["SheetContent", "content", "native <dialog> opened with showModal(), role=dialog, data-side, data-overlay"],
        &["", "panel", "The sliding panel against the chosen edge"],
        &["SheetHeader", "header", "Title, description and close icon"],
        &["SheetTitle", "title", "h2, labels the dialog"],
        &["SheetDescription", "description", "p, describes the dialog"],
        &["SheetBody", "body", "Main content. Scrolls when it overflows"],
        &["SheetFooter", "footer", "Right-aligned button row"],
        &["SheetClose", "close", "button that closes the sheet"],
        &["SheetCloseIcon", "close-icon", "Icon-only close button, aria-label=Close"],
    ],
    keys: &[
        &["Escape", "Closes the sheet"],
        &["Tab / Shift+Tab", "Moves focus within the sheet; the page behind is inert"],
        &["Enter / Space", "Activates the focused button"],
    ],
    props: &[
        &["open", "Option<bool>", "Controlled open state. Pair with on_open_change"],
        &["default_open", "bool", "Initial open state when uncontrolled"],
        &["on_open_change", "Callback<bool>", "Called when the sheet opens or closes"],
        &["side", "SheetSide", "On SheetContent: Start, End, Top or Bottom. Default End"],
        &["variant", "Option<ButtonVariant>", "On SheetTrigger and SheetClose: gives the button look"],
    ],
    events: &[],
};
