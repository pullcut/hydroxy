//! Attachment connector. Anatomy: root, thumbnail, badge, body, name, meta,
//! remove. Every part carries `data-scope="attachment"` and its `data-part`;
//! the root also carries `data-variant` (`chip` or `card`) and `data-kind`.
//! It only displays a file, so there is no machine: the remove button is native.

pub use crate::file_dropzone::format_size;

/// The `data-scope` value on every attachment part.
pub const SCOPE: &str = "attachment";

/// How it's laid out.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum Variant {
    /// One compact line, to sit inside a composer or a message.
    #[default]
    Chip,
    /// A bigger tile with room for a thumbnail.
    Card,
}

impl Variant {
    /// The `data-variant` value.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Variant::Chip => "chip",
            Variant::Card => "card",
        }
    }
}

/// What sort of file it is, which picks the badge colour and the type label.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Kind {
    /// Pictures.
    Image,
    /// PDF.
    Pdf,
    /// Word processing, plain text and notes.
    Document,
    /// Spreadsheets and CSV.
    Spreadsheet,
    /// Slides.
    Presentation,
    /// Zip, tar and friends.
    Archive,
    /// Sound.
    Audio,
    /// Video.
    Video,
    /// Source and data files.
    Code,
    /// Anything else.
    Other,
}

impl Kind {
    /// The `data-kind` value.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Kind::Image => "image",
            Kind::Pdf => "pdf",
            Kind::Document => "document",
            Kind::Spreadsheet => "spreadsheet",
            Kind::Presentation => "presentation",
            Kind::Archive => "archive",
            Kind::Audio => "audio",
            Kind::Video => "video",
            Kind::Code => "code",
            Kind::Other => "other",
        }
    }

    /// A short phrase for people: `"PDF"`, `"Spreadsheet"`, `"File"`.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Kind::Image => "Image",
            Kind::Pdf => "PDF",
            Kind::Document => "Document",
            Kind::Spreadsheet => "Spreadsheet",
            Kind::Presentation => "Presentation",
            Kind::Archive => "Archive",
            Kind::Audio => "Audio",
            Kind::Video => "Video",
            Kind::Code => "Code",
            Kind::Other => "File",
        }
    }
}

/// The file's extension, lower case, without the dot. Empty when there is none.
#[must_use]
pub fn extension(name: &str) -> String {
    match name.rsplit_once('.') {
        Some((stem, ext)) if !stem.is_empty() => ext.to_ascii_lowercase(),
        _ => String::new(),
    }
}

/// Classify a file by its MIME type, falling back to the extension.
#[must_use]
pub fn kind_of(name: &str, mime: &str) -> Kind {
    let mime = mime.to_ascii_lowercase();
    let by_mime = if mime.starts_with("image/") {
        Some(Kind::Image)
    } else if mime.starts_with("audio/") {
        Some(Kind::Audio)
    } else if mime.starts_with("video/") {
        Some(Kind::Video)
    } else if mime == "application/pdf" {
        Some(Kind::Pdf)
    } else {
        None
    };
    by_mime.unwrap_or_else(|| match extension(name).as_str() {
        "png" | "jpg" | "jpeg" | "gif" | "webp" | "svg" | "avif" | "heic" => Kind::Image,
        "pdf" => Kind::Pdf,
        "doc" | "docx" | "txt" | "md" | "rtf" | "odt" | "pages" => Kind::Document,
        "xls" | "xlsx" | "csv" | "tsv" | "ods" | "numbers" => Kind::Spreadsheet,
        "ppt" | "pptx" | "key" | "odp" => Kind::Presentation,
        "zip" | "tar" | "gz" | "tgz" | "7z" | "rar" => Kind::Archive,
        "mp3" | "wav" | "m4a" | "flac" | "ogg" => Kind::Audio,
        "mp4" | "mov" | "webm" | "mkv" | "avi" => Kind::Video,
        "rs" | "js" | "ts" | "json" | "html" | "css" | "py" | "toml" | "yaml" | "yml" | "sh" => Kind::Code,
        _ => Kind::Other,
    })
}

/// The text inside the badge: the upper-case extension, at most four characters.
#[must_use]
pub fn badge_text(name: &str) -> String {
    let ext = extension(name);
    if ext.is_empty() { "FILE".to_string() } else { ext.chars().take(4).collect::<String>().to_uppercase() }
}

/// The line under the name: `"PDF, 2.4 MB"`, or just `"PDF"` without a size.
#[must_use]
pub fn meta_text(kind: Kind, size: Option<u64>) -> String {
    match size {
        Some(bytes) => format!("{}, {}", kind.label(), format_size(bytes)),
        None => kind.label().to_string(),
    }
}

/// The remove button's `aria-label`.
#[must_use]
pub fn remove_label(name: &str) -> String {
    format!("Remove {name}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mime_wins_then_extension() {
        assert_eq!(kind_of("photo", "image/png"), Kind::Image);
        assert_eq!(kind_of("report.PDF", ""), Kind::Pdf);
        assert_eq!(kind_of("data.csv", "text/csv"), Kind::Spreadsheet);
        assert_eq!(kind_of("notes", ""), Kind::Other);
        assert_eq!(kind_of(".gitignore", ""), Kind::Other);
    }

    #[test]
    fn badge_and_meta_text() {
        assert_eq!(badge_text("report.final.pdf"), "PDF");
        assert_eq!(badge_text("archive.tar.gzip"), "GZIP");
        assert_eq!(badge_text("README"), "FILE");
        assert_eq!(meta_text(Kind::Pdf, Some(2_516_582)), "PDF, 2.4 MB");
        assert_eq!(meta_text(Kind::Other, None), "File");
        assert_eq!(remove_label("a.pdf"), "Remove a.pdf");
    }
}
