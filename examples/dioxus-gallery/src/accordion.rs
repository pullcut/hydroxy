use dioxus::prelude::*;
use hydroxy_dioxus::accordion::{Accordion, AccordionContent, AccordionEntry, AccordionItem, AccordionTrigger};

#[component]
pub fn AccordionDemo() -> Element {
    let mut open = use_signal(|| vec!["sources".to_string()]);

    rsx! {
        // Uncontrolled, single: opening one item closes the other.
        Accordion {
            items: vec![
                AccordionEntry::new("citing"),
                AccordionEntry::new("files"),
                AccordionEntry::new("sharing").disabled(),
                AccordionEntry::new("export"),
            ],
            default_value: vec!["citing".to_string()],
            AccordionItem { value: "citing",
                AccordionTrigger { "How sources are cited" }
                AccordionContent { "Every answer lists the files it read, with the page for each figure." }
            }
            AccordionItem { value: "files",
                AccordionTrigger { "What happens to uploaded files" }
                AccordionContent { "Files stay in the project they were added to. Deleting the project deletes them." }
            }
            AccordionItem { value: "sharing",
                AccordionTrigger { "Team sharing" }
                AccordionContent { "Not available on this plan." }
            }
            AccordionItem { value: "export",
                AccordionTrigger { "Exporting a summary" }
                AccordionContent { "Summaries export as Markdown or PDF, sources included." }
            }
        }

        // Controlled, multiple: several items open at once.
        div {
            p { "Open: {open().join(\", \")}" }
            Accordion {
                items: vec![AccordionEntry::new("sources"), AccordionEntry::new("checks")],
                value: Some(open()),
                on_value_change: move |v| open.set(v),
                multiple: true,
                AccordionItem { value: "sources",
                    AccordionTrigger { "Sources" }
                    AccordionContent { "interviews-q3.pdf, tickets-sept.csv" }
                }
                AccordionItem { value: "checks",
                    AccordionTrigger { "Checks" }
                    AccordionContent { "22 of 24 passed. Two need your judgement." }
                }
            }
        }
    }
}
