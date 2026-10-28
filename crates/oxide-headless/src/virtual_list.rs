//! Virtual list connector. Anatomy:
//!
//! - `viewport`: the scroll container and `role="listbox"`. It holds focus; the active
//!   option is announced through `aria-activedescendant`.
//! - `spacer`: sized to the whole list so the scrollbar is true. Rows sit inside it.
//! - `group`: one per rendered group, spanning its rows (`role="group"`, labelled by its
//!   header). Rows before the first header sit in an unlabelled group box.
//! - `group-label`: the header, `position: sticky` inside its group.
//! - `item`: one rendered option (`role="option"`), positioned with a transform.
//!   `aria-setsize`/`aria-posinset` give its place in the whole list, not just the
//!   rendered slice. `data-active` and `data-selected` mirror the state.
//!
//! Every part carries `data-scope="virtual-list"` and its `data-part`. Rows and headers
//! carry `data-index` so a size observer can map an element back to its row.

pub use oxide_machines::virtual_list::*;

/// `data-scope` for every part.
pub const SCOPE: &str = "virtual-list";

/// The id of row `index` (option or header).
#[must_use]
pub fn item_id(root: &str, index: usize) -> String {
    format!("{root}-item-{index}")
}

/// `aria-activedescendant` on the viewport.
#[must_use]
pub fn activedescendant(root: &str, state: &VirtualListState) -> Option<String> {
    state.active.map(|i| item_id(root, i))
}

/// Attributes for one option.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemAttrs {
    /// `id`, for `aria-activedescendant`.
    pub id: String,
    /// `aria-selected`.
    pub selected: bool,
    /// `data-active`: the keyboard is on it.
    pub active: bool,
    /// `aria-setsize`: options in the whole list.
    pub aria_setsize: usize,
    /// `aria-posinset`, from 1.
    pub aria_posinset: usize,
}

/// Attributes for option `index`.
#[must_use]
pub fn item_attrs(root: &str, state: &VirtualListState, input: &VirtualListInput, index: usize) -> ItemAttrs {
    ItemAttrs {
        id: item_id(root, index),
        selected: state.selected == Some(index),
        active: state.active == Some(index),
        aria_setsize: option_count(&input.headers, input.count),
        aria_posinset: pos_in_set(&input.headers, index),
    }
}

/// `style` for the spacer: the full height of the list.
#[must_use]
pub fn spacer_style(layout: &Layout) -> String {
    format!("height: {}px", layout.total())
}

/// `style` for a group box.
#[must_use]
pub fn block_style(block: &Block) -> String {
    format!("top: {}px; height: {}px", block.top, block.height)
}

/// `style` for a row inside a group box starting at `block_top`. `fixed` is the row's
/// own size when sizes are fixed rather than measured.
#[must_use]
pub fn row_style(layout: &Layout, block_top: f64, index: usize, fixed: bool) -> String {
    let y = layout.offset(index) - block_top;
    if fixed {
        format!("transform: translateY({y}px); height: {}px", layout.size(index))
    } else {
        format!("transform: translateY({y}px)")
    }
}

/// `style` for a header when sizes are fixed; `None` when measured.
#[must_use]
pub fn header_style(layout: &Layout, index: usize, fixed: bool) -> Option<String> {
    fixed.then(|| format!("height: {}px", layout.size(index)))
}

/// Rows a Page key moves: one less than a viewport's worth, so one row of context stays.
#[must_use]
pub fn page_rows(layout: &Layout, scroll: f64, viewport: f64) -> usize {
    layout.visible(scroll, viewport).len().saturating_sub(1).max(1)
}

/// Map a `KeyboardEvent.key` on the listbox to an event. `Some` means: prevent default.
#[must_use]
pub fn keydown(key: &str, page: usize) -> Option<VirtualListEvent> {
    let nav = match key {
        "ArrowDown" => Nav::Next,
        "ArrowUp" => Nav::Prev,
        "Home" => Nav::First,
        "End" => Nav::Last,
        "PageDown" => Nav::PageDown(page),
        "PageUp" => Nav::PageUp(page),
        "Enter" | " " => return Some(VirtualListEvent::SelectActive),
        _ => return None,
    };
    Some(VirtualListEvent::Move(nav))
}

/// Where to scroll so row `index` shows below its group's sticky header.
#[must_use]
pub fn scroll_target(layout: &Layout, headers: &[usize], index: usize, align: Align, scroll: f64, viewport: f64) -> f64 {
    layout.scroll_to(index, align, scroll, viewport, sticky_inset(layout, headers, index))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn attrs_place_options_in_the_whole_list() {
        let input = VirtualListInput { count: 100_000, headers: vec![0, 50], default_selected: None };
        let state = VirtualListState { active: Some(51), selected: Some(51) };
        let a = item_attrs("ox-vl-0", &state, &input, 51);
        assert_eq!(a.id, "ox-vl-0-item-51");
        assert_eq!(a.aria_setsize, 99_998);
        assert_eq!(a.aria_posinset, 50);
        assert!(a.selected && a.active);
        assert_eq!(activedescendant("ox-vl-0", &state).as_deref(), Some("ox-vl-0-item-51"));
    }

    #[test]
    fn keys_map_to_navigation() {
        assert_eq!(keydown("PageDown", 9), Some(VirtualListEvent::Move(Nav::PageDown(9))));
        assert_eq!(keydown(" ", 9), Some(VirtualListEvent::SelectActive));
        assert_eq!(keydown("a", 9), None);
    }

    #[test]
    fn styles_and_paging() {
        let l = Layout::fixed(100, 20.0);
        assert_eq!(spacer_style(&l), "height: 2000px");
        assert_eq!(row_style(&l, 100.0, 7, true), "transform: translateY(40px); height: 20px");
        assert_eq!(row_style(&l, 0.0, 1, false), "transform: translateY(20px)");
        assert_eq!(page_rows(&l, 0.0, 200.0), 9);
        assert_eq!(page_rows(&l, 0.0, 0.0), 1);
        let hs = [0];
        assert!((scroll_target(&l, &hs, 10, Align::Start, 0.0, 200.0) - 180.0).abs() < 1e-9);
    }
}
