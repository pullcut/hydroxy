//! Lightbox connector. Anatomy: content (a native `<dialog>` opened with
//! `showModal()`, which traps focus, closes on Escape and gives focus back on
//! close), header (counter and actions), counter (`aria-live`), stage (the
//! zoom and swipe surface), media (the `<img>` or `<video>`), caption, prev,
//! next, close, zoom, download, thumbs (the strip) and thumb. Every part
//! carries `data-scope="lightbox"`.

pub use hydroxy_machines::lightbox::*;

/// The `data-scope` value on every part.
pub const SCOPE: &str = "lightbox";

/// Image or video.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum MediaKind {
    /// A still image: zoomable.
    #[default]
    Image,
    /// A video with native controls.
    Video,
}

/// One item in a lightbox or gallery.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MediaItem {
    /// Full-size source.
    pub src: String,
    /// Smaller source for the grid and the strip. Falls back to `src` (or `poster` for video).
    pub thumb: Option<String>,
    /// Text alternative. Required for images.
    pub alt: String,
    /// Shown under the media.
    pub caption: Option<String>,
    /// Image or video.
    pub kind: MediaKind,
    /// A video's still frame.
    pub poster: Option<String>,
    /// Where the download button points. `None` hides it.
    pub download: Option<String>,
}

impl MediaItem {
    /// An image.
    #[must_use]
    pub fn image(src: impl Into<String>, alt: impl Into<String>) -> Self {
        Self { src: src.into(), alt: alt.into(), ..Self::default() }
    }

    /// A video.
    #[must_use]
    pub fn video(src: impl Into<String>, alt: impl Into<String>) -> Self {
        Self { src: src.into(), alt: alt.into(), kind: MediaKind::Video, ..Self::default() }
    }

    /// Use a smaller source in grids and the strip.
    #[must_use]
    pub fn thumb(mut self, thumb: impl Into<String>) -> Self {
        self.thumb = Some(thumb.into());
        self
    }

    /// Add a caption.
    #[must_use]
    pub fn caption(mut self, caption: impl Into<String>) -> Self {
        self.caption = Some(caption.into());
        self
    }

    /// A video's still frame.
    #[must_use]
    pub fn poster(mut self, poster: impl Into<String>) -> Self {
        self.poster = Some(poster.into());
        self
    }

    /// Offer a download of `href`.
    #[must_use]
    pub fn download(mut self, href: impl Into<String>) -> Self {
        self.download = Some(href.into());
        self
    }

    /// The small source for grids and the strip.
    #[must_use]
    pub fn thumb_src(&self) -> &str {
        self.thumb
            .as_deref()
            .or_else(|| self.poster.as_deref().filter(|_| self.kind == MediaKind::Video))
            .unwrap_or(&self.src)
    }

    /// Whether it can zoom.
    #[must_use]
    pub fn is_image(&self) -> bool {
        self.kind == MediaKind::Image
    }
}

/// Ids for one lightbox.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ids {
    /// The `<dialog>`.
    pub content: String,
    /// The stage.
    pub stage: String,
    /// The counter, which labels the dialog together with the caption.
    pub counter: String,
}

/// Ids derived from one root id.
#[must_use]
pub fn ids(root: &str) -> Ids {
    Ids { content: format!("{root}-content"), stage: format!("{root}-stage"), counter: format!("{root}-counter") }
}

/// "3 of 12".
#[must_use]
pub fn counter(index: usize, count: usize) -> String {
    format!("{} of {count}", index + 1)
}

/// A thumbnail button's name: "Show 3: Harbour at dusk".
#[must_use]
pub fn thumb_label(index: usize, item: &MediaItem) -> String {
    format!("Show {}: {}", index + 1, item.alt)
}

/// The items to preload: the neighbours of `index` (images only).
#[must_use]
pub fn neighbours(index: usize, count: usize, looping: bool) -> Vec<usize> {
    let mut out = Vec::new();
    if count < 2 {
        return out;
    }
    let prev = if index == 0 { looping.then_some(count - 1) } else { Some(index - 1) };
    let next = if index + 1 == count { looping.then_some(0) } else { Some(index + 1) };
    out.extend(prev);
    out.extend(next.filter(|n| Some(*n) != prev));
    out
}

/// Map a key in the dialog to an event, given the image's current size.
/// Escape is left to the native dialog. `Some` means: prevent default.
#[must_use]
pub fn keydown(key: &str, size: Size) -> Option<LightboxEvent> {
    match key {
        "ArrowRight" => Some(LightboxEvent::Next),
        "ArrowLeft" => Some(LightboxEvent::Prev),
        "Home" => Some(LightboxEvent::First),
        "End" => Some(LightboxEvent::Last),
        "+" | "=" => Some(LightboxEvent::ZoomIn(size)),
        "-" | "_" => Some(LightboxEvent::ZoomOut(size)),
        "0" => Some(LightboxEvent::ResetZoom),
        _ => None,
    }
}

/// Watches the stage. A JS function body with parameters `root` (the stage
/// id) and `send` (called with a string); returns a cleanup function. Reports
/// `"left,top,width,height,imageWidth,imageHeight"`: the stage's box, and the
/// media's unzoomed size, whenever either changes or an image finishes loading.
pub const WATCH_JS: &str = r#"
const stage = document.getElementById(root);
if (!stage) return () => {};
const report = () => {
  const r = stage.getBoundingClientRect();
  const m = stage.querySelector('[data-part="media"]');
  send([r.left, r.top, r.width, r.height, m ? m.offsetWidth : 0, m ? m.offsetHeight : 0].map((n) => n.toFixed(1)).join(","));
};
const ro = new ResizeObserver(report);
ro.observe(stage);
const onload = () => report();
stage.addEventListener("load", onload, true);
stage.addEventListener("loadedmetadata", onload, true);
report();
return () => { ro.disconnect(); stage.removeEventListener("load", onload, true); stage.removeEventListener("loadedmetadata", onload, true); };
"#;

/// The stage box and media size from a [`WATCH_JS`] report.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct StageBox {
    /// Viewport x of the stage's left edge.
    pub left: f64,
    /// Viewport y of the stage's top edge.
    pub top: f64,
    /// Stage width.
    pub width: f64,
    /// Stage height.
    pub height: f64,
    /// The media's unzoomed size.
    pub media: Size,
}

/// A [`WATCH_JS`] report that is not six comma-separated numbers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BadStageReport;

impl std::fmt::Display for BadStageReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("stage report is not six comma-separated numbers")
    }
}

impl std::error::Error for BadStageReport {}

/// Parses a [`WATCH_JS`] report.
impl std::str::FromStr for StageBox {
    type Err = BadStageReport;

    fn from_str(report: &str) -> Result<Self, BadStageReport> {
        let n: Vec<f64> =
            report.split(',').map(|s| s.trim().parse().ok()).collect::<Option<_>>().ok_or(BadStageReport)?;
        let [left, top, width, height, w, h] = n[..] else { return Err(BadStageReport) };
        Ok(Self { left, top, width, height, media: Size { w, h } })
    }
}

impl StageBox {
    /// A viewport point relative to the stage centre.
    #[must_use]
    pub fn point(&self, client_x: f64, client_y: f64) -> Point {
        Point::new(client_x - self.left - self.width / 2.0, client_y - self.top - self.height / 2.0)
    }
}

/// Scrolls the strip so the current thumbnail shows. A JS function body with
/// the parameter `root` (the dialog id).
pub const SHOW_THUMB_JS: &str = r#"
const t = document.getElementById(root)?.querySelector('[data-part="thumb"][aria-current="true"]');
t?.scrollIntoView({ block: "nearest", inline: "center", behavior: matchMedia("(prefers-reduced-motion: reduce)").matches ? "auto" : "smooth" });
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn neighbours_respect_ends_and_looping() {
        assert_eq!(neighbours(0, 5, false), vec![1]);
        assert_eq!(neighbours(0, 5, true), vec![4, 1]);
        assert_eq!(neighbours(4, 5, false), vec![3]);
        assert_eq!(neighbours(1, 2, true), vec![0]);
        assert!(neighbours(0, 1, true).is_empty());
    }

    #[test]
    fn keys_map_to_events() {
        let size = Size { w: 10.0, h: 10.0 };
        assert_eq!(keydown("ArrowRight", size), Some(LightboxEvent::Next));
        assert_eq!(keydown("+", size), Some(LightboxEvent::ZoomIn(size)));
        assert_eq!(keydown("0", size), Some(LightboxEvent::ResetZoom));
        assert_eq!(keydown("Escape", size), None);
    }

    #[test]
    fn stage_points_are_centred() {
        let b = "10,20,200,100,150,80".parse::<StageBox>().unwrap();
        assert_eq!(b.media, Size { w: 150.0, h: 80.0 });
        assert_eq!(b.point(110.0, 70.0), Point::new(0.0, 0.0));
        assert!("1,2".parse::<StageBox>().is_err());
    }

    #[test]
    fn video_thumbs_fall_back_to_the_poster() {
        let v = MediaItem::video("a.mp4", "Clip").poster("a.jpg");
        assert_eq!(v.thumb_src(), "a.jpg");
        assert_eq!(MediaItem::image("b.jpg", "B").thumb_src(), "b.jpg");
        assert_eq!(counter(2, 12), "3 of 12");
    }
}
