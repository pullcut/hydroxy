//! File dropzone connector. Anatomy: root, dropzone (a `<label>` that takes
//! drops), input (the native `<input type="file">` inside it, visually
//! hidden), icon, label, hint, error, list, item, name, size and remove.
//! Every part carries `data-scope="file-dropzone"` and its `data-part`; the
//! dropzone carries `data-state="dragging"` or `"idle"`.
//!
//! The label opens the picker on click and the input itself holds focus, so
//! Enter and Space open it natively; no key mapping or script is needed.

pub use oxide_machines::file_dropzone::*;

/// `data-scope` for every part.
pub const SCOPE: &str = "file-dropzone";

/// `data-state` for the dropzone.
#[must_use]
pub fn data_state<F>(state: &FileDropzoneState<F>) -> &'static str {
    if state.dragging() { "dragging" } else { "idle" }
}

/// A byte count for people: `"820 B"`, `"1.4 KB"`, `"3.2 MB"`.
#[must_use]
#[allow(clippy::cast_precision_loss, reason = "display only")]
pub fn format_size(bytes: u64) -> String {
    const UNITS: [&str; 4] = ["KB", "MB", "GB", "TB"];
    if bytes < 1024 {
        return format!("{bytes} B");
    }
    let mut size = bytes as f64 / 1024.0;
    let mut unit = 0;
    while size >= 1024.0 && unit < UNITS.len() - 1 {
        size /= 1024.0;
        unit += 1;
    }
    format!("{size:.1} {}", UNITS[unit])
}

/// One line per rejection, such as `"notes.txt: File type not allowed"`.
#[must_use]
pub fn rejection_text<F>(rejections: &[Rejection<F>]) -> Vec<String> {
    rejections.iter().map(|r| format!("{}: {}", r.file.name, r.reason.message())).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sizes_read_well() {
        assert_eq!(format_size(820), "820 B");
        assert_eq!(format_size(1434), "1.4 KB");
        assert_eq!(format_size(3_355_443), "3.2 MB");
    }

    #[test]
    fn rejection_lines() {
        let file = PickedFile { name: "a.txt".into(), size: 1, mime: String::new(), file: () };
        assert_eq!(rejection_text(&[Rejection { file, reason: RejectReason::FileType }]), ["a.txt: File type not allowed"]);
    }
}
