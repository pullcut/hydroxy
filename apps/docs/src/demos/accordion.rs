use hydroxy_leptos::accordion::*;
use leptos::prelude::*;

pub fn demo() -> AnyView {
    let (open, set_open) = signal(vec!["sources".to_string()]);

    view! {
        // Uncontrolled, single: opening one item closes the other.
        <Accordion
            items=vec![
                AccordionEntry::new("citing"),
                AccordionEntry::new("files"),
                AccordionEntry::new("sharing").disabled(),
                AccordionEntry::new("export"),
            ]
            default_value=vec!["citing".to_string()]
        >
            <AccordionItem value="citing">
                <AccordionTrigger>"How sources are cited"</AccordionTrigger>
                <AccordionContent>"Every answer lists the files it read, with the page for each figure."</AccordionContent>
            </AccordionItem>
            <AccordionItem value="files">
                <AccordionTrigger>"What happens to uploaded files"</AccordionTrigger>
                <AccordionContent>"Files stay in the project they were added to. Deleting the project deletes them."</AccordionContent>
            </AccordionItem>
            <AccordionItem value="sharing">
                <AccordionTrigger>"Team sharing"</AccordionTrigger>
                <AccordionContent>"Not available on this plan."</AccordionContent>
            </AccordionItem>
            <AccordionItem value="export">
                <AccordionTrigger>"Exporting a summary"</AccordionTrigger>
                <AccordionContent>"Summaries export as Markdown or PDF, sources included."</AccordionContent>
            </AccordionItem>
        </Accordion>

        // Controlled, multiple: several items open at once.
        <div>
            <p>"Open: " {move || open.get().join(", ")}</p>
            <Accordion
                items=vec![AccordionEntry::new("sources"), AccordionEntry::new("checks")]
                value=open
                on_value_change=move |v| set_open.set(v)
                multiple=true
            >
                <AccordionItem value="sources">
                    <AccordionTrigger>"Sources"</AccordionTrigger>
                    <AccordionContent>"interviews-q3.pdf, tickets-sept.csv"</AccordionContent>
                </AccordionItem>
                <AccordionItem value="checks">
                    <AccordionTrigger>"Checks"</AccordionTrigger>
                    <AccordionContent>"22 of 24 passed. Two need your judgement."</AccordionContent>
                </AccordionItem>
            </Accordion>
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "accordion",
    name: "Accordion",
    group: "Disclosure",
    summary: "Stacked sections that open and close. Single mode keeps one open; multiple mode opens each on its own.",
    demo,
    leptos: include_str!("accordion.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/accordion.rs"),
    parts: &[
        &["Accordion", "root", "Provides the machine. data-orientation=vertical"],
        &["AccordionItem", "item", "Wraps one section. data-state=open|closed, data-disabled"],
        &["AccordionTrigger", "heading, trigger", "h3 around a button: aria-expanded, aria-controls, data-state"],
        &["AccordionContent", "content", "role=region, aria-labelledby, hidden when closed"],
    ],
    keys: &[
        &["Enter / Space", "Opens or closes the focused item"],
        &["ArrowDown / ArrowUp", "Next / previous enabled trigger. Wraps at the ends"],
        &["Home / End", "First / last enabled trigger"],
        &["Tab", "Moves through triggers and open content in page order"],
    ],
    props: &[
        &["items", "Vec<AccordionEntry>", "Required. Order and disabled state"],
        &["value", "Option<Vec<String>>", "Controlled open items. Pair with on_value_change"],
        &["default_value", "Vec<String>", "Initially open items when uncontrolled"],
        &["multiple", "bool", "Allow several items open at once. Default false"],
        &["collapsible", "bool", "Single mode: the open item can close. Default true"],
        &["on_value_change", "Callback<Vec<String>>", "Called when the user opens or closes an item"],
    ],
    events: &[],
};
