//! Upload queue connector. Anatomy: root, summary, list, item, name, meta,
//! progress (a [`ProgressBar`](crate::progress_bar)), actions, action, live.
//! Every part carries `data-scope="upload-queue"` and its `data-part`; an item
//! carries `data-state` (`queued`, `uploading`, `paused`, `done`, `failed`,
//! `cancelled`). Items are plain `<li>`s and the actions are native buttons.

use std::fmt::Write;

pub use hydroxy_machines::upload_queue::*;

pub use crate::file_dropzone::format_size;

/// The `data-scope` value on every upload queue part.
pub const SCOPE: &str = "upload-queue";

/// One button on an item row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    /// Hold it.
    Pause,
    /// Continue a paused item.
    Resume,
    /// Stop it.
    Cancel,
    /// Start a failed or cancelled item again.
    Retry,
    /// Take it off the list.
    Remove,
}

impl Action {
    /// The `data-action` value.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Action::Pause => "pause",
            Action::Resume => "resume",
            Action::Cancel => "cancel",
            Action::Retry => "retry",
            Action::Remove => "remove",
        }
    }

    /// The short visible word for the button.
    #[must_use]
    pub const fn word(self) -> &'static str {
        match self {
            Action::Pause => "Pause",
            Action::Resume => "Resume",
            Action::Cancel => "Cancel",
            Action::Retry => "Retry",
            Action::Remove => "Remove",
        }
    }

    /// `aria-label` text, with the file name so each button is unique.
    #[must_use]
    pub fn label(self, name: &str) -> String {
        format!("{} {name}", self.word())
    }

    /// The event this button sends for item `id`.
    #[must_use]
    pub fn event(self, id: &str) -> UploadQueueEvent {
        let id = id.to_string();
        match self {
            Action::Pause => UploadQueueEvent::Pause(id),
            Action::Resume => UploadQueueEvent::Resume(id),
            Action::Cancel => UploadQueueEvent::Cancel(id),
            Action::Retry => UploadQueueEvent::Retry(id),
            Action::Remove => UploadQueueEvent::Remove(id),
        }
    }
}

/// The buttons an item shows, in order.
#[must_use]
pub fn actions(item: &UploadItem) -> Vec<Action> {
    match item.status {
        UploadStatus::Queued | UploadStatus::Uploading => vec![Action::Pause, Action::Cancel],
        UploadStatus::Paused => vec![Action::Resume, Action::Cancel],
        UploadStatus::Failed | UploadStatus::Cancelled => vec![Action::Retry, Action::Remove],
        UploadStatus::Done => vec![Action::Remove],
    }
}

/// The line under the file name: `"Uploading, 40% of 1.0 KB"`.
#[must_use]
pub fn status_text(item: &UploadItem) -> String {
    let size = format_size(item.size);
    match item.status {
        UploadStatus::Queued => format!("Waiting, {size}"),
        UploadStatus::Uploading => format!("Uploading, {}% of {size}", item.percent()),
        UploadStatus::Paused => format!("Paused at {}%, {size}", item.percent()),
        UploadStatus::Done => format!("Done, {size}"),
        UploadStatus::Failed => format!("Failed: {}", item.error.as_deref().unwrap_or("Upload failed")),
        UploadStatus::Cancelled => "Cancelled".to_string(),
    }
}

/// The heading line: `"2 of 4 uploaded, 1 failed"`.
#[must_use]
pub fn summary_text(items: &[UploadItem]) -> String {
    let s = summarize(items);
    if s.total == 0 {
        return "No uploads".to_string();
    }
    let mut text = format!("{} of {} uploaded", s.done, s.total);
    if s.failed > 0 {
        let _ = write!(text, ", {} failed", s.failed);
    }
    text
}

/// What to say in the live region for a command, if anything.
#[must_use]
pub fn announcement(command: &UploadQueueCommand, items: &[UploadItem]) -> Option<String> {
    let name = |id: &str| items.iter().find(|i| i.id == id).map_or_else(|| id.to_string(), |i| i.name.clone());
    match command {
        UploadQueueCommand::Completed(id) => Some(format!("{} uploaded.", name(id))),
        UploadQueueCommand::Failed { id, message } => Some(format!("{} failed: {message}.", name(id))),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(status: UploadStatus, sent: u64) -> UploadItem {
        UploadItem { status, sent, ..UploadItem::new(UploadFile::new("a", "a.pdf", 2048)) }
    }

    #[test]
    fn actions_follow_the_status() {
        assert_eq!(actions(&item(UploadStatus::Uploading, 0)), [Action::Pause, Action::Cancel]);
        assert_eq!(actions(&item(UploadStatus::Paused, 0)), [Action::Resume, Action::Cancel]);
        assert_eq!(actions(&item(UploadStatus::Failed, 0)), [Action::Retry, Action::Remove]);
        assert_eq!(actions(&item(UploadStatus::Done, 0)), [Action::Remove]);
        assert_eq!(Action::Retry.event("a"), UploadQueueEvent::Retry("a".into()));
        assert_eq!(Action::Pause.label("a.pdf"), "Pause a.pdf");
    }

    #[test]
    fn text_reads_well() {
        assert_eq!(status_text(&item(UploadStatus::Uploading, 1024)), "Uploading, 50% of 2.0 KB");
        let mut failed = item(UploadStatus::Failed, 0);
        failed.error = Some("Network lost".into());
        assert_eq!(status_text(&failed), "Failed: Network lost");
        assert_eq!(summary_text(&[item(UploadStatus::Done, 0), failed]), "1 of 2 uploaded, 1 failed");
        assert_eq!(summary_text(&[]), "No uploads");
    }

    #[test]
    fn only_finishes_are_announced() {
        let items = [item(UploadStatus::Done, 0)];
        assert_eq!(
            announcement(&UploadQueueCommand::Completed("a".into()), &items).as_deref(),
            Some("a.pdf uploaded.")
        );
        assert_eq!(announcement(&UploadQueueCommand::Abort("a".into()), &items), None);
    }
}
