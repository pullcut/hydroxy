use hydroxy_leptos::tabs::*;
use leptos::prelude::*;

pub fn demo() -> AnyView {
    let (value, set_value) = signal(Some("overview".to_string()));

    view! {
        // Controlled, horizontal, automatic activation.
        <Tabs
            tabs=vec![
                TabsItem::new("overview"),
                TabsItem::new("sources"),
                TabsItem::new("archived").disabled(),
                TabsItem::new("activity"),
            ]
            value=value
            on_value_change=move |v| set_value.set(Some(v))
        >
            <TabsList>
                <TabsTrigger value="overview">"Overview"</TabsTrigger>
                <TabsTrigger value="sources">"Sources"</TabsTrigger>
                <TabsTrigger value="archived">"Archived"</TabsTrigger>
                <TabsTrigger value="activity">"Activity"</TabsTrigger>
            </TabsList>
            <TabsContent value="overview">"Nine of twelve interviewees raised onboarding."</TabsContent>
            <TabsContent value="sources">"interviews-q3.pdf, tickets-sept.csv"</TabsContent>
            <TabsContent value="archived">"Nothing archived."</TabsContent>
            <TabsContent value="activity">"Summary drafted 26 October."</TabsContent>
        </Tabs>

        // Uncontrolled, vertical, manual activation: arrows move focus, Enter selects.
        <Tabs
            tabs=vec![TabsItem::new("general"), TabsItem::new("members"), TabsItem::new("billing")]
            default_value="general"
            orientation=Orientation::Vertical
            activation=Activation::Manual
        >
            <div style="display: flex; gap: var(--space-8);">
                <TabsList>
                    <TabsTrigger value="general">"General"</TabsTrigger>
                    <TabsTrigger value="members">"Members"</TabsTrigger>
                    <TabsTrigger value="billing">"Billing"</TabsTrigger>
                </TabsList>
                <TabsContent value="general">"Workspace name and default theme."</TabsContent>
                <TabsContent value="members">"22 of 24 seats in use."</TabsContent>
                <TabsContent value="billing">"Next invoice 1 November."</TabsContent>
            </div>
        </Tabs>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "tabs",
    name: "Tabs",
    group: "Navigation",
    summary: "Switch between views within a page. One tab is selected; arrow keys move between enabled tabs.",
    demo,
    leptos: include_str!("tabs.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/tabs.rs"),
    parts: &[
        &["Tabs", "root", "Provides the machine. data-orientation"],
        &["TabsList", "list", "role=tablist, aria-orientation"],
        &["TabsTrigger", "trigger", "role=tab, aria-selected, aria-controls, data-state=active|inactive"],
        &["TabsContent", "content", "role=tabpanel, aria-labelledby, hidden when inactive"],
    ],
    keys: &[
        &["ArrowRight / ArrowLeft", "Next / previous enabled tab (horizontal). Selects it with automatic activation"],
        &["ArrowDown / ArrowUp", "Next / previous enabled tab (vertical)"],
        &["Home / End", "First / last enabled tab"],
        &["Enter / Space", "Selects the focused tab (manual activation)"],
        &["Tab", "Moves focus into the selected panel"],
    ],
    props: &[
        &["tabs", "Vec<TabsItem>", "Required. Order and disabled state"],
        &["value", "Option<String>", "Controlled value. Pair with on_value_change"],
        &["default_value", "String", "Initial value when uncontrolled"],
        &["activation", "Activation", "Automatic (default) or Manual"],
        &["orientation", "Orientation", "Horizontal (default) or Vertical"],
        &["loop_focus", "bool", "Wrap at the ends. Default true"],
        &["on_value_change", "Callback<String>", "Called when the user selects a tab"],
    ],
};
