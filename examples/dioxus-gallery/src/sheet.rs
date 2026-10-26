use dioxus::prelude::*;
use oxide_dioxus::button::ButtonVariant;
use oxide_dioxus::sheet::{
    Sheet, SheetBody, SheetClose, SheetCloseIcon, SheetContent, SheetDescription, SheetFooter, SheetHeader, SheetSide,
    SheetTitle, SheetTrigger,
};

#[component]
pub fn SheetDemo() -> Element {
    let mut filters_open = use_signal(|| Some(false));

    rsx! {
        div { style: "display: flex; flex-wrap: wrap; gap: var(--space-3);",
            // Uncontrolled, from the inline end edge (right in LTR).
            Sheet {
                SheetTrigger { variant: ButtonVariant::Primary, "Project details" }
                SheetContent {
                    SheetHeader {
                        SheetTitle { "Q3 customer research" }
                        SheetDescription { "12 files, 3 summaries. Updated 2 hours ago." }
                        SheetCloseIcon {}
                    }
                    SheetBody {
                        p { "Interviews with 18 customers across retail and logistics, grouped by theme." }
                        p { "Summaries cite their sources, so every claim links back to a transcript." }
                    }
                    SheetFooter {
                        SheetClose { variant: ButtonVariant::Secondary, "Close" }
                        SheetClose { variant: ButtonVariant::Accent, "Open project" }
                    }
                }
            }

            // Controlled, from the bottom edge.
            Sheet {
                open: filters_open(),
                on_open_change: move |open| filters_open.set(Some(open)),
                SheetTrigger { variant: ButtonVariant::Secondary, "Filters" }
                SheetContent { side: SheetSide::Bottom,
                    SheetHeader {
                        SheetTitle { "Filter sources" }
                        SheetCloseIcon {}
                    }
                    SheetBody {
                        p { "Show transcripts, notes and summaries from the last 90 days." }
                    }
                    SheetFooter {
                        SheetClose { variant: ButtonVariant::Primary, "Apply filters" }
                    }
                }
            }
        }
    }
}
