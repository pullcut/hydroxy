//! Shared overlay groundwork for tooltips, menus and dialogs.
//!
//! hydroxy leans on the platform instead of re-implementing it:
//! - **Top layer and light dismiss**: the Popover API (`popover` attribute).
//! - **Modal focus trap and Esc**: `<dialog>` opened with `showModal()`, which makes
//!   the rest of the page inert.
//! - **Positioning and flipping**: CSS anchor positioning. The styles below tie a
//!   floating element to its anchor and flip it to the other side when it would
//!   overflow the viewport.

/// Side of the anchor the floating element sits on. Logical, so it follows `dir`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum Side {
    /// Above the anchor.
    Top,
    /// Below the anchor.
    #[default]
    Bottom,
    /// Before the anchor (left in LTR).
    Start,
    /// After the anchor (right in LTR).
    End,
}

/// How the floating element lines up with the anchor along the other axis.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum Align {
    /// Line up with the anchor's start edge.
    Start,
    /// Centre on the anchor.
    #[default]
    Center,
    /// Line up with the anchor's end edge.
    End,
}

/// Where a floating element goes relative to its anchor.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Placement {
    /// See [`Side`].
    pub side: Side,
    /// See [`Align`].
    pub align: Align,
}

impl Placement {
    /// A placement from its parts.
    #[must_use]
    pub const fn new(side: Side, align: Align) -> Self {
        Self { side, align }
    }

    /// The CSS `position-area` value.
    #[must_use]
    pub const fn position_area(self) -> &'static str {
        match (self.side, self.align) {
            (Side::Top, Align::Start) => "block-start span-inline-end",
            (Side::Top, Align::Center) => "block-start center",
            (Side::Top, Align::End) => "block-start span-inline-start",
            (Side::Bottom, Align::Start) => "block-end span-inline-end",
            (Side::Bottom, Align::Center) => "block-end center",
            (Side::Bottom, Align::End) => "block-end span-inline-start",
            (Side::Start, Align::Start) => "inline-start span-block-end",
            (Side::Start, Align::Center) => "inline-start center",
            (Side::Start, Align::End) => "inline-start span-block-start",
            (Side::End, Align::Start) => "inline-end span-block-end",
            (Side::End, Align::Center) => "inline-end center",
            (Side::End, Align::End) => "inline-end span-block-start",
        }
    }

    /// The `data-side` value, for side-aware styling such as the enter animation.
    #[must_use]
    pub const fn side_str(self) -> &'static str {
        match self.side {
            Side::Top => "top",
            Side::Bottom => "bottom",
            Side::Start => "start",
            Side::End => "end",
        }
    }

    /// The margin property facing the anchor, and the flip that keeps it on screen.
    const fn gap_and_flip(self) -> (&'static str, &'static str) {
        match self.side {
            Side::Top => ("margin-block-end", "flip-block"),
            Side::Bottom => ("margin-block-start", "flip-block"),
            Side::Start => ("margin-inline-end", "flip-inline"),
            Side::End => ("margin-inline-start", "flip-inline"),
        }
    }
}

/// Inline style for the anchor (trigger) element.
#[must_use]
pub fn anchor_style(id: &str) -> String {
    format!("anchor-name: --{id};")
}

/// Inline style for the floating element: fixed to the anchor named by `id`,
/// `gap_px` away from it, flipping to the other side when it would overflow.
#[must_use]
pub fn floating_style(id: &str, placement: Placement, gap_px: u16) -> String {
    let (gap, flip) = placement.gap_and_flip();
    format!(
        "position: fixed; inset: auto; margin: 0; position-anchor: --{id}; position-area: {}; {gap}: {gap_px}px; position-try-fallbacks: {flip};",
        placement.position_area()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn floating_style_names_the_anchor_and_gap_side() {
        let style = floating_style("ox-menu-1", Placement::new(Side::Bottom, Align::Start), 6);
        assert!(style.contains("position-anchor: --ox-menu-1;"));
        assert!(style.contains("position-area: block-end span-inline-end;"));
        assert!(style.contains("margin-block-start: 6px;"));
        assert!(style.contains("position-try-fallbacks: flip-block;"));
        assert_eq!(anchor_style("ox-menu-1"), "anchor-name: --ox-menu-1;");
    }

    #[test]
    fn side_placements_flip_inline() {
        let style = floating_style("t", Placement::new(Side::End, Align::Center), 8);
        assert!(style.contains("margin-inline-start: 8px;"));
        assert!(style.contains("flip-inline"));
    }
}
