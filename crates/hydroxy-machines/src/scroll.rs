//! Scroll-to-bottom maths for lists that follow their newest entry.

/// Distance in px from the bottom edge: `scroll_height - scroll_top - client_height`,
/// never negative.
#[must_use]
pub fn from_bottom(scroll_height: f64, scroll_top: f64, client_height: f64) -> f64 {
    (scroll_height - scroll_top - client_height).max(0.0)
}

/// Whether the view is within `threshold` px of the bottom.
#[must_use]
pub fn at_bottom(scroll_height: f64, scroll_top: f64, client_height: f64, threshold: f64) -> bool {
    from_bottom(scroll_height, scroll_top, client_height) <= threshold
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn measures_distance_to_the_bottom() {
        assert!((from_bottom(1000.0, 400.0, 500.0) - 100.0).abs() < f64::EPSILON);
        assert!(from_bottom(1000.0, 520.0, 500.0).abs() < f64::EPSILON);
        assert!(at_bottom(1000.0, 495.0, 500.0, 8.0));
        assert!(!at_bottom(1000.0, 400.0, 500.0, 8.0));
    }
}
