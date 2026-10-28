use hydroxy_leptos::activity::{ActivityFeed, ActivityItem};
use hydroxy_leptos::avatar::Jewel;
use hydroxy_leptos::value_diff::ValueDiff;
use leptos::prelude::*;

pub fn demo() -> AnyView {
    view! {
        <div style="max-width: 520px;">
            <ActivityFeed>
                <ActivityItem name="Jo Marsh" initials="JM" jewel=Jewel::Sapphire time="2h">
                    "moved the status" <ValueDiff from="In review" to="Ready" />
                </ActivityItem>
                <ActivityItem name="hydroxy" initials="hy" agent=true time="1h">
                    "drafted a summary from 12 files"
                </ActivityItem>
                <ActivityItem name="Amara Kaur" initials="AK" jewel=Jewel::Emerald time="40m" comment=true>
                    "Two of the interviews contradict the August summary. Can we check the September export first?"
                </ActivityItem>
                <ActivityItem name="Rui Tavares" initials="RT" jewel=Jewel::Garnet time="12m">
                    "changed the due date" <ValueDiff from="24 Oct" to="31 Oct" />
                </ActivityItem>
            </ActivityFeed>
        </div>
    }
    .into_any()
}

// @docs: page data below, hidden from the code sample.

use crate::doc::Doc;

pub(crate) const DOC: Doc = Doc {
    slug: "activity-feed",
    name: "Activity feed",
    group: "Activity",
    summary: "A timeline of changes and comments. Each item is a line, often with a value diff, or a bordered comment.",
    demo,
    leptos: include_str!("activity.rs"),
    dioxus: include_str!("../../../../examples/dioxus-gallery/src/activity.rs"),
    parts: &[
        &["ActivityFeed", "—", "ol[data-scope=activity-feed] with a rule through the avatars"],
        &["ActivityItem", "item", "li: avatar, then a line or a comment"],
        &["", "time", "When, in mono"],
    ],
    keys: &[],
    props: &[
        &["name", "String", "ActivityItem. Required. Who did it"],
        &["time", "String", "ActivityItem. Required. Such as 2h or 26 Oct"],
        &["initials", "String", "ActivityItem. Avatar initials"],
        &["jewel", "Jewel", "ActivityItem. Avatar colour"],
        &["agent", "bool", "ActivityItem. Draw the avatar as an agent"],
        &["comment", "bool", "ActivityItem. Show children as a comment bubble"],
        &["label", "String", "ActivityFeed. Accessible name. Default Activity"],
    ],
};
