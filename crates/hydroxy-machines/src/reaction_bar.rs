//! Reaction bar: emoji reactions on a message, each with a count and whether you
//! are among the reactors. The host owns the reactions; this module words them
//! and works out the toggles.

/// One reaction on a message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reaction {
    /// The emoji, such as "👍".
    pub emoji: String,
    /// Its name, for screen readers, such as "thumbs up".
    pub name: String,
    /// Everyone else who reacted with it, in reaction order.
    pub people: Vec<String>,
    /// You reacted with it.
    pub reacted: bool,
}

impl Reaction {
    /// A reaction from its emoji, name and the other people.
    #[must_use]
    pub fn new(emoji: impl Into<String>, name: impl Into<String>, people: &[&str]) -> Self {
        Self {
            emoji: emoji.into(),
            name: name.into(),
            people: people.iter().map(|p| (*p).to_string()).collect(),
            reacted: false,
        }
    }

    /// Mark that you reacted too.
    #[must_use]
    pub fn mine(mut self) -> Self {
        self.reacted = true;
        self
    }

    /// Everyone, you included.
    #[must_use]
    pub fn count(&self) -> usize {
        self.people.len() + usize::from(self.reacted)
    }
}

/// Who reacted, for the tooltip: "You, Ana and Ben reacted with thumbs up".
/// Names at most `max` people and counts the rest.
#[must_use]
pub fn who(reaction: &Reaction, max: usize) -> String {
    let mut names: Vec<&str> = Vec::with_capacity(reaction.count());
    if reaction.reacted {
        names.push("You");
    }
    names.extend(reaction.people.iter().map(String::as_str));
    if names.is_empty() {
        return format!("No one reacted with {}", reaction.name);
    }
    format!("{} reacted with {}", crate::presence_bar::list(&names, max), reaction.name)
}

/// The chip's accessible name: "thumbs up, 3 reactions". The pressed state is
/// carried by `aria-pressed`.
#[must_use]
pub fn chip_label(reaction: &Reaction) -> String {
    let n = reaction.count();
    let noun = if n == 1 { "reaction" } else { "reactions" };
    format!("{}, {n} {noun}", reaction.name)
}

/// What to announce after you toggle: "Added thumbs up" or "Removed thumbs up".
#[must_use]
pub fn toggle_announcement(reaction: &Reaction) -> String {
    // Called with the reaction as it was before the toggle.
    let verb = if reaction.reacted { "Removed" } else { "Added" };
    format!("{verb} {}", reaction.name)
}

/// Apply a toggle of `emoji` by you to `reactions`, as the host would: flips your
/// reaction, drops a reaction nobody holds any more, or appends a new one.
/// A convenience for hosts that keep reactions locally.
pub fn toggle(reactions: &mut Vec<Reaction>, emoji: &str, name: &str) {
    match reactions.iter().position(|r| r.emoji == emoji) {
        Some(i) => {
            reactions[i].reacted = !reactions[i].reacted;
            if reactions[i].count() == 0 {
                reactions.remove(i);
            }
        }
        None => reactions.push(Reaction::new(emoji, name, &[]).mine()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_include_you() {
        let r = Reaction::new("👍", "thumbs up", &["Ana", "Ben"]).mine();
        assert_eq!(r.count(), 3);
        assert_eq!(chip_label(&r), "thumbs up, 3 reactions");
        assert_eq!(chip_label(&Reaction::new("🎉", "party popper", &["Ana"])), "party popper, 1 reaction");
    }

    #[test]
    fn who_names_you_first_and_counts_the_rest() {
        let r = Reaction::new("👍", "thumbs up", &["Ana", "Ben", "Cy", "Dee"]).mine();
        assert_eq!(who(&r, 3), "You, Ana and 3 others reacted with thumbs up");
        assert_eq!(who(&Reaction::new("👀", "eyes", &["Ana"]), 3), "Ana reacted with eyes");
    }

    #[test]
    fn toggling_adds_flips_and_removes() {
        let mut list = vec![Reaction::new("👍", "thumbs up", &["Ana"])];
        toggle(&mut list, "👍", "thumbs up");
        assert!(list[0].reacted);
        assert_eq!(toggle_announcement(&list[0]), "Removed thumbs up");
        toggle(&mut list, "🎉", "party popper");
        assert_eq!(list.len(), 2);
        toggle(&mut list, "🎉", "party popper");
        assert_eq!(list.len(), 1, "a reaction nobody holds goes away");
    }
}
