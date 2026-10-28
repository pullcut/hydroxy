//! Word clouds: heaviest words first, each walked out along a golden-angle spiral from
//! the centre until its box clears every word already placed. Deterministic: the
//! same words always land in the same places. Boxes are estimated from character
//! counts, not measured glyphs, so leave a little padding.

/// A word and its weight.
#[derive(Debug, Clone, PartialEq)]
pub struct Word {
    /// The word or short phrase.
    pub text: String,
    /// Its weight, such as a count. Sets the font size.
    pub weight: f64,
}

impl Word {
    /// A weighted word.
    #[must_use]
    pub fn new(text: impl Into<String>, weight: f64) -> Self {
        Self { text: text.into(), weight }
    }
}

/// A placed word, in the cloud's px box.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Placed {
    /// Index into the input.
    pub index: usize,
    /// Centre x.
    pub x: f64,
    /// Centre y (the text's middle, for `dominant-baseline: central`).
    pub y: f64,
    /// Font size in px.
    pub font_size: f64,
    /// Estimated box width, padding included.
    pub width: f64,
    /// Estimated box height, padding included.
    pub height: f64,
}

/// Layout settings.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Options {
    /// Box width in px.
    pub width: f64,
    /// Box height in px.
    pub height: f64,
    /// Font size for the lightest word.
    pub min_font: f64,
    /// Font size for the heaviest word.
    pub max_font: f64,
    /// Clear space around each word.
    pub padding: f64,
}

impl Default for Options {
    fn default() -> Self {
        Self { width: 520.0, height: 300.0, min_font: 12.0, max_font: 44.0, padding: 2.0 }
    }
}

const GOLDEN: f64 = 2.399_963_229_728_653;
const ATTEMPTS: u32 = 2400;

fn lcg(state: &mut u32) -> f64 {
    *state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
    f64::from(*state) / 4_294_967_296.0
}

#[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation, reason = "word counts and lengths are small")]
/// Place `words`, heaviest first. Words that find no room are left out, so the
/// result can be shorter than the input. Sorted by weight, heaviest first.
#[must_use]
pub fn layout(words: &[Word], opts: Options) -> Vec<Placed> {
    let weight = |word: &Word| if word.weight.is_finite() { word.weight } else { 0.0 };
    let lo = words.iter().map(weight).fold(f64::INFINITY, f64::min);
    let hi = words.iter().map(weight).fold(f64::NEG_INFINITY, f64::max);
    let mut order: Vec<usize> = (0..words.len()).collect();
    order.sort_by(|&a, &b| weight(&words[b]).total_cmp(&weight(&words[a])).then(a.cmp(&b)));
    let (cx, cy) = (opts.width / 2.0, opts.height / 2.0);
    let aspect = opts.width / opts.height.max(1.0);
    let mut placed: Vec<Placed> = Vec::new();
    for i in order {
        let t = if hi > lo { (weight(&words[i]) - lo) / (hi - lo) } else { 0.5 };
        let font_size = opts.min_font + t * (opts.max_font - opts.min_font);
        let chars = words[i].text.trim().chars().count().max(1) as f64;
        let (bw, bh) = (chars * font_size * 0.58 + 2.0 * opts.padding, font_size + 2.0 * opts.padding);
        let mut seed = (i as u32).wrapping_mul(9973).wrapping_add(1);
        // O(n²) box checks; a spatial grid if clouds grow past a few hundred words.
        let spot = (0..ATTEMPTS).find_map(|k| {
            let angle = (f64::from(k) + lcg(&mut seed) * 0.25) * GOLDEN;
            let r = 3.0 * f64::from(k).sqrt();
            let (x, y) = (cx + angle.cos() * r * aspect.sqrt(), cy + angle.sin() * r / aspect.sqrt());
            let inside =
                x - bw / 2.0 >= 0.0 && x + bw / 2.0 <= opts.width && y - bh / 2.0 >= 0.0 && y + bh / 2.0 <= opts.height;
            let clear =
                placed.iter().all(|p| (p.x - x).abs() * 2.0 >= p.width + bw || (p.y - y).abs() * 2.0 >= p.height + bh);
            (inside && clear).then_some((x, y))
        });
        if let Some((x, y)) = spot {
            placed.push(Placed { index: i, x, y, font_size, width: bw, height: bh });
        }
    }
    placed
}

#[cfg(test)]
mod tests {
    use super::*;

    fn words() -> Vec<Word> {
        ["latency", "search", "billing", "export", "sso", "dark mode", "api", "mobile", "alerts", "onboarding"]
            .iter()
            .zip(0..)
            .map(|(t, k)| Word::new(*t, 50.0 - 4.0 * f64::from(k)))
            .collect()
    }

    #[test]
    fn heaviest_word_sits_at_the_centre_at_the_largest_size() {
        let p = layout(&words(), Options::default());
        assert_eq!(p[0].index, 0);
        assert_eq!((p[0].x, p[0].y, p[0].font_size), (260.0, 150.0, 44.0));
        assert_eq!(p.last().map(|w| w.font_size), Some(12.0));
    }

    #[test]
    fn no_two_boxes_overlap_and_all_stay_inside() {
        let opts = Options::default();
        let p = layout(&words(), opts);
        assert_eq!(p.len(), 10);
        for (i, a) in p.iter().enumerate() {
            assert!(a.x - a.width / 2.0 >= 0.0 && a.x + a.width / 2.0 <= opts.width);
            for b in &p[i + 1..] {
                assert!((a.x - b.x).abs() * 2.0 >= a.width + b.width || (a.y - b.y).abs() * 2.0 >= a.height + b.height);
            }
        }
    }

    #[test]
    fn layout_is_deterministic_and_drops_what_cannot_fit() {
        assert_eq!(layout(&words(), Options::default()), layout(&words(), Options::default()));
        let tiny = Options { width: 60.0, height: 30.0, ..Options::default() };
        assert!(layout(&words(), tiny).len() < 10);
    }
}
