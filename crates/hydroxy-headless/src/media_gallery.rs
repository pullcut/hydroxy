//! Media gallery connector. Anatomy: root (a `<ul>` grid), item (`<li>`), thumb
//! (the `<button>` that opens the lightbox), image, and badge (the play mark on
//! videos). Every part carries `data-scope="media-gallery"`. The lightbox it
//! opens uses the [`lightbox`](crate::lightbox) connector.

pub use crate::lightbox::{MediaItem, MediaKind};

/// The `data-scope` value on every part.
pub const SCOPE: &str = "media-gallery";

/// The id of thumbnail `index`.
#[must_use]
pub fn thumb_id(root: &str, index: usize) -> String {
    format!("{root}-thumb-{index}")
}

/// A thumbnail button's name: "Open image: Harbour at dusk" or "Play video: …".
#[must_use]
pub fn thumb_label(item: &MediaItem) -> String {
    match item.kind {
        MediaKind::Image => format!("Open image: {}", item.alt),
        MediaKind::Video => format!("Play video: {}", item.alt),
    }
}

/// Grows the open lightbox's media out of the thumbnail it was opened from.
/// A JS function body with the parameter `thumb` (the thumbnail's id). Does
/// nothing when the user prefers reduced motion.
pub const ZOOM_FROM_JS: &str = r#"
if (matchMedia("(prefers-reduced-motion: reduce)").matches) return;
const from = document.getElementById(thumb);
const run = (tries) => {
  const to = document.querySelector('dialog[data-scope="lightbox"][open] [data-part="media"]');
  if (!from || !to) return;
  const a = from.getBoundingClientRect(), b = to.getBoundingClientRect();
  if (!b.width || !b.height) { if (tries > 0) requestAnimationFrame(() => run(tries - 1)); return; }
  const root = getComputedStyle(document.documentElement);
  const ms = parseFloat(root.getPropertyValue("--duration-slow")) || 320;
  const ease = root.getPropertyValue("--ease-out").trim() || "ease-out";
  const dx = a.left + a.width / 2 - (b.left + b.width / 2), dy = a.top + a.height / 2 - (b.top + b.height / 2);
  const s = Math.max(a.width / b.width, a.height / b.height);
  to.animate([{ transform: `translate(${dx}px, ${dy}px) scale(${s})`, opacity: 0.6 }, { transform: "none", opacity: 1 }], { duration: ms, easing: ease });
};
requestAnimationFrame(() => run(30));
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn labels_say_what_opens() {
        assert_eq!(thumb_label(&MediaItem::image("a", "Harbour")), "Open image: Harbour");
        assert_eq!(thumb_label(&MediaItem::video("a", "Clip")), "Play video: Clip");
        assert_eq!(thumb_id("g", 3), "g-thumb-3");
    }
}
