//! Every oxide component in one desktop window. Run with `cargo run -p dioxus-gallery`.
//! Each module is also shown as the Dioxus code on its docs page.

#![allow(missing_docs, clippy::must_use_candidate, reason = "example app")]

use dioxus::prelude::*;

mod accordion;
mod avatar;
mod badge;
mod button;
mod callout;
mod card;
mod checkbox;
mod chip;
mod collapsible;
mod combobox;
mod command_palette;
mod dialog;
mod empty_state;
mod icon_button;
mod input;
mod kbd;
mod menu;
mod popover;
mod progress_bar;
mod radio_group;
mod search_field;
mod segmented;
mod select;
mod slider;
mod switch;
mod tabs;
mod tag;
mod tags_input;
mod toast;
mod tooltip;

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        style { {oxide_tokens::CSS} }
        main { style: "max-width: 720px; margin: 0 auto; padding: 32px; display: grid; gap: 48px;",
            section { h2 { "Tabs" } tabs::TabsDemo {} }
            section { h2 { "Select" } select::SelectDemo {} }
            section { h2 { "Combobox" } combobox::ComboboxDemo {} }
            section { h2 { "Command palette" } command_palette::CommandPaletteDemo {} }
            section { h2 { "Popover" } popover::PopoverDemo {} }
            section { h2 { "Toast" } toast::ToastDemo {} }
            section { h2 { "Slider" } slider::SliderDemo {} }
            section { h2 { "Segmented control" } segmented::SegmentedDemo {} }
            section { h2 { "Tags input" } tags_input::TagsInputDemo {} }
            section { h2 { "Button" } button::ButtonDemo {} }
            section { h2 { "IconButton" } icon_button::IconButtonDemo {} }
            section { h2 { "Badge" } badge::BadgeDemo {} }
            section { h2 { "Tag" } tag::TagDemo {} }
            section { h2 { "Chip" } chip::ChipDemo {} }
            section { h2 { "Kbd" } kbd::KbdDemo {} }
            section { h2 { "Avatar" } avatar::AvatarDemo {} }
            section { h2 { "Card" } card::CardDemo {} }
            section { h2 { "Input" } input::InputDemo {} }
            section { h2 { "SearchField" } search_field::SearchFieldDemo {} }
            section { h2 { "Callout" } callout::CalloutDemo {} }
            section { h2 { "EmptyState" } empty_state::EmptyStateDemo {} }
            section { h2 { "ProgressBar" } progress_bar::ProgressBarDemo {} }
            section { h2 { "Dialog" } dialog::DialogDemo {} }
            section { h2 { "Menu" } menu::MenuDemo {} }
            section { h2 { "Tooltip" } tooltip::TooltipDemo {} }
            section { h2 { "Checkbox" } checkbox::CheckboxDemo {} }
            section { h2 { "Switch" } switch::SwitchDemo {} }
            section { h2 { "RadioGroup" } radio_group::RadioGroupDemo {} }
            section { h2 { "Collapsible" } collapsible::CollapsibleDemo {} }
            section { h2 { "Accordion" } accordion::AccordionDemo {} }
        }
    }
}
