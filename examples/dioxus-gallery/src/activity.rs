use dioxus::prelude::*;
use oxide_dioxus::activity::{ActivityFeed, ActivityItem};
use oxide_dioxus::avatar::Jewel;
use oxide_dioxus::value_diff::ValueDiff;

#[component]
pub fn ActivityFeedDemo() -> Element {
    rsx! {
        div { style: "max-width: 520px;",
            ActivityFeed {
                ActivityItem { name: "Jo Marsh", initials: "JM", jewel: Jewel::Sapphire, time: "2h",
                    "moved the status"
                    ValueDiff { from: "In review", to: "Ready" }
                }
                ActivityItem { name: "oxide", initials: "ox", agent: true, time: "1h",
                    "drafted a summary from 12 files"
                }
                ActivityItem { name: "Amara Kaur", initials: "AK", jewel: Jewel::Emerald, time: "40m", comment: true,
                    "Two of the interviews contradict the August summary. Can we check the September export first?"
                }
                ActivityItem { name: "Rui Tavares", initials: "RT", jewel: Jewel::Garnet, time: "12m",
                    "changed the due date"
                    ValueDiff { from: "24 Oct", to: "31 Oct" }
                }
            }
        }
    }
}
