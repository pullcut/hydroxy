//! Presence bar connector. Anatomy: root (`role="group"` named by the summary),
//! stack (`<ul>`), item (`<li>`, focusable, with an avatar, a status dot and a
//! name tip), overflow (a button that opens the overflow list), list (a native
//! `popover="auto"` anchored to the overflow button). Every part carries
//! `data-scope="presence-bar"` and its `data-part`; items carry `data-status`.

pub use oxide_machines::presence_bar::*;

use crate::jewel::Jewel;
use crate::overlay::{Align, Placement, Side, anchor_style, floating_style};

/// The `data-scope` value on every part.
pub const SCOPE: &str = "presence-bar";

/// One person in the room.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Collaborator {
    /// Stable id.
    pub id: String,
    /// Display name.
    pub name: String,
    /// How present they are.
    pub status: PresenceStatus,
    /// Avatar tint. `None` picks one from the id.
    pub jewel: Option<Jewel>,
    /// This is you.
    pub is_self: bool,
}

impl Collaborator {
    /// An active collaborator.
    #[must_use]
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self { id: id.into(), name: name.into(), status: PresenceStatus::Active, jewel: None, is_self: false }
    }

    /// The same person with this status.
    #[must_use]
    pub fn status(mut self, status: PresenceStatus) -> Self {
        self.status = status;
        self
    }

    /// The same person with this tint.
    #[must_use]
    pub fn jewel(mut self, jewel: Jewel) -> Self {
        self.jewel = Some(jewel);
        self
    }

    /// Mark as you.
    #[must_use]
    pub fn me(mut self) -> Self {
        self.is_self = true;
        self
    }

    /// Their tint: the chosen one, or a stable pick from the id.
    #[must_use]
    pub fn tint(&self) -> Jewel {
        self.jewel.unwrap_or_else(|| jewel_for(&self.id))
    }

    /// Their accessible name, such as "Ana Lima (you), idle".
    #[must_use]
    pub fn label(&self) -> String {
        person_label(&self.name, self.status, self.is_self)
    }
}

/// A stable tint for an id.
#[must_use]
pub fn jewel_for(id: &str) -> Jewel {
    const ORDER: [Jewel; 6] =
        [Jewel::Amethyst, Jewel::Sapphire, Jewel::Emerald, Jewel::Garnet, Jewel::Citrine, Jewel::Rhodolite];
    ORDER[colour_slot(id)]
}

/// The bar's layout: people in display order, split at the overflow.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Layout {
    /// Shown as avatars.
    pub visible: Vec<Collaborator>,
    /// Behind the "+N" button.
    pub overflow: Vec<Collaborator>,
    /// The group's accessible name.
    pub summary: String,
}

/// Order `people` and split them for `max` avatar slots.
#[must_use]
pub fn layout(people: &[Collaborator], max: usize) -> Layout {
    let statuses: Vec<PresenceStatus> = people.iter().map(|p| p.status).collect();
    let ordered: Vec<Collaborator> = order(&statuses).into_iter().map(|i| people[i].clone()).collect();
    let names: Vec<&str> = ordered.iter().map(|p| p.name.as_str()).collect();
    let summary = summary(&names);
    let shown = visible_count(ordered.len(), max);
    let mut visible = ordered;
    let overflow = visible.split_off(shown);
    Layout { visible, overflow, summary }
}

/// Element ids for one presence bar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ids {
    /// The overflow button; also the CSS anchor name.
    pub overflow: String,
    /// The overflow list popover.
    pub list: String,
}

/// Ids derived from one root id.
#[must_use]
pub fn ids(root: &str) -> Ids {
    Ids { overflow: format!("{root}-overflow"), list: format!("{root}-list") }
}

/// Inline style naming the overflow button as the list's anchor.
#[must_use]
pub fn overflow_style(ids: &Ids) -> String {
    anchor_style(&ids.overflow)
}

/// Inline style placing the overflow list below the button, end-aligned.
#[must_use]
pub fn list_style(ids: &Ids) -> String {
    floating_style(&ids.overflow, Placement::new(Side::Bottom, Align::End), 8)
}

/// Inline style naming one avatar as its name tip's anchor.
#[must_use]
pub fn item_style(root: &str, person: &str) -> String {
    anchor_style(&format!("{root}-p-{person}"))
}

/// Inline style placing an avatar's name tip below it.
#[must_use]
pub fn tip_style(root: &str, person: &str) -> String {
    floating_style(&format!("{root}-p-{person}"), Placement::new(Side::Bottom, Align::Center), 6)
}

/// The name on an avatar's tip: "Ana Lima", or "Ana Lima (you)".
#[must_use]
pub fn display_name(person: &Collaborator) -> String {
    if person.is_self { format!("{} (you)", person.name) } else { person.name.clone() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layout_orders_and_splits() {
        let people = vec![
            Collaborator::new("a", "Ana").status(PresenceStatus::Away),
            Collaborator::new("b", "Ben"),
            Collaborator::new("c", "Cy"),
            Collaborator::new("d", "Dee").status(PresenceStatus::Idle),
            Collaborator::new("e", "Eve"),
        ];
        let l = layout(&people, 4);
        let names = |v: &[Collaborator]| v.iter().map(|p| p.name.clone()).collect::<Vec<_>>();
        assert_eq!(names(&l.visible), ["Ben", "Cy", "Eve"]);
        assert_eq!(names(&l.overflow), ["Dee", "Ana"]);
        assert_eq!(l.summary, "5 people here: Ben, Cy, Eve and 2 others");
    }

    #[test]
    fn tint_falls_back_to_the_id() {
        assert_eq!(Collaborator::new("x", "X").tint(), jewel_for("x"));
        assert_eq!(Collaborator::new("x", "X").jewel(Jewel::Garnet).tint(), Jewel::Garnet);
        let i = ids("pb");
        assert!(list_style(&i).contains("position-anchor: --pb-overflow;"));
        assert!(tip_style("pb", "x").contains("position-anchor: --pb-p-x;"));
        assert_eq!(display_name(&Collaborator::new("x", "X").me()), "X (you)");
    }
}
