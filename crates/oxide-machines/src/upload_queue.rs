//! Upload queue owns the list of uploads and what each is doing. It never
//! moves bytes: it asks the owner to [`UploadQueueCommand::Start`] or
//! [`Abort`](UploadQueueCommand::Abort) a transfer, and the owner reports
//! back with [`UploadQueueEvent::Progress`], `Complete` or `Fail`. At most
//! `concurrency` items upload at once; the rest wait in the queue.

use oxide_statechart::{Machine, Transition};

/// Where an upload is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UploadStatus {
    /// Waiting for a free slot.
    Queued,
    /// Transferring. The owner reports progress.
    Uploading,
    /// Held by the user. Resume puts it back in the queue.
    Paused,
    /// Finished.
    Done,
    /// The transfer failed. Retry starts it again.
    Failed,
    /// Stopped by the user. Retry starts it again.
    Cancelled,
}

impl UploadStatus {
    /// The `data-state` value.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            UploadStatus::Queued => "queued",
            UploadStatus::Uploading => "uploading",
            UploadStatus::Paused => "paused",
            UploadStatus::Done => "done",
            UploadStatus::Failed => "failed",
            UploadStatus::Cancelled => "cancelled",
        }
    }

    /// No further progress is expected without a user action.
    #[must_use]
    pub const fn is_settled(self) -> bool {
        matches!(self, UploadStatus::Done | UploadStatus::Failed | UploadStatus::Cancelled)
    }
}

/// A file to add. The owner keeps the file handle and looks it up by `id`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UploadFile {
    /// Unique key. Adding an id that's already queued does nothing.
    pub id: String,
    /// File name.
    pub name: String,
    /// Size in bytes.
    pub size: u64,
}

impl UploadFile {
    /// A file.
    #[must_use]
    pub fn new(id: impl Into<String>, name: impl Into<String>, size: u64) -> Self {
        Self { id: id.into(), name: name.into(), size }
    }
}

/// One row in the queue.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UploadItem {
    /// Unique key.
    pub id: String,
    /// File name.
    pub name: String,
    /// Size in bytes.
    pub size: u64,
    /// Where it is.
    pub status: UploadStatus,
    /// Bytes sent so far.
    pub sent: u64,
    /// Why it failed, once it has.
    pub error: Option<String>,
    /// How many times it has been started.
    pub attempt: u32,
}

impl UploadItem {
    /// A queued item with nothing sent.
    #[must_use]
    pub fn new(file: UploadFile) -> Self {
        Self {
            id: file.id,
            name: file.name,
            size: file.size,
            status: UploadStatus::Queued,
            sent: 0,
            error: None,
            attempt: 0,
        }
    }

    /// Whole percent sent, 0–100. An empty file counts as done once it is.
    #[must_use]
    pub fn percent(&self) -> u8 {
        if self.size == 0 {
            return if self.status == UploadStatus::Done { 100 } else { 0 };
        }
        let pct = u128::from(self.sent.min(self.size)) * 100 / u128::from(self.size);
        u8::try_from(pct).unwrap_or(100)
    }
}

/// Counts for a summary line.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct UploadSummary {
    /// All items.
    pub total: usize,
    /// Queued or paused.
    pub waiting: usize,
    /// Uploading now.
    pub uploading: usize,
    /// Done.
    pub done: usize,
    /// Failed.
    pub failed: usize,
    /// Cancelled.
    pub cancelled: usize,
}

/// Count the items by status.
#[must_use]
pub fn summarize(items: &[UploadItem]) -> UploadSummary {
    let mut s = UploadSummary { total: items.len(), ..UploadSummary::default() };
    for item in items {
        match item.status {
            UploadStatus::Queued | UploadStatus::Paused => s.waiting += 1,
            UploadStatus::Uploading => s.uploading += 1,
            UploadStatus::Done => s.done += 1,
            UploadStatus::Failed => s.failed += 1,
            UploadStatus::Cancelled => s.cancelled += 1,
        }
    }
    s
}

/// Props for [`UploadQueueMachine`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UploadQueueInput {
    /// Most transfers at once. Zero counts as one.
    pub concurrency: usize,
}

impl Default for UploadQueueInput {
    fn default() -> Self {
        Self { concurrency: 3 }
    }
}

/// What [`UploadQueueMachine`] remembers between events.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct UploadQueueState {
    /// The rows, in the order they were added.
    pub items: Vec<UploadItem>,
}

/// Inputs to [`UploadQueueMachine`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UploadQueueEvent {
    /// Queue these files.
    Add(Vec<UploadFile>),
    /// The owner sent `sent` bytes of `id` so far.
    Progress {
        /// The item.
        id: String,
        /// Bytes sent so far.
        sent: u64,
    },
    /// The owner finished sending `id`.
    Complete(String),
    /// The transfer of `id` failed.
    Fail {
        /// The item.
        id: String,
        /// Shown to the user.
        message: String,
    },
    /// Hold a queued or uploading item.
    Pause(String),
    /// Put a paused item back in the queue.
    Resume(String),
    /// Stop a queued, uploading or paused item.
    Cancel(String),
    /// Queue a failed or cancelled item again, from the start.
    Retry(String),
    /// Drop an item from the list, aborting it if it's uploading.
    Remove(String),
    /// Drop every done and cancelled item.
    ClearFinished,
}

/// Side effects from [`UploadQueueMachine`] for the adapter to run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UploadQueueCommand {
    /// Begin sending `id`, from byte `from` (non-zero after a resume).
    Start {
        /// The item.
        id: String,
        /// Attempt number, starting at 1.
        attempt: u32,
        /// Bytes already sent.
        from: u64,
    },
    /// Stop sending `id`.
    Abort(String),
    /// `id` is no longer in the list. Drop its file handle.
    Removed(String),
    /// `id` finished.
    Completed(String),
    /// `id` failed with this message.
    Failed {
        /// The item.
        id: String,
        /// Why.
        message: String,
    },
}

/// Pending uploads and their progress.
#[derive(Debug, Clone, Copy)]
pub struct UploadQueueMachine;

type Step = Transition<UploadQueueState, UploadQueueCommand>;

/// Start queued items while there are free slots.
fn schedule(items: &mut [UploadItem], input: &UploadQueueInput, commands: &mut Vec<UploadQueueCommand>) {
    let running = items.iter().filter(|i| i.status == UploadStatus::Uploading).count();
    let free = input.concurrency.max(1).saturating_sub(running);
    for item in items.iter_mut().filter(|i| i.status == UploadStatus::Queued).take(free) {
        item.status = UploadStatus::Uploading;
        item.attempt += 1;
        item.error = None;
        commands.push(UploadQueueCommand::Start { id: item.id.clone(), attempt: item.attempt, from: item.sent });
    }
}

impl Machine for UploadQueueMachine {
    type Input = UploadQueueInput;
    type State = UploadQueueState;
    type Event = UploadQueueEvent;
    type Command = UploadQueueCommand;

    fn initial(_input: &UploadQueueInput) -> UploadQueueState {
        UploadQueueState::default()
    }

    fn transition(state: &UploadQueueState, event: UploadQueueEvent, input: &UploadQueueInput) -> Step {
        let mut items = state.items.clone();
        let mut commands = Vec::new();
        let find = |items: &[UploadItem], id: &str| items.iter().position(|i| i.id == id);
        match event {
            UploadQueueEvent::Add(files) => {
                for file in files {
                    if find(&items, &file.id).is_none() {
                        items.push(UploadItem::new(file));
                    }
                }
            }
            UploadQueueEvent::Progress { id, sent } => {
                let Some(at) = find(&items, &id) else { return Transition::stay(state.clone()) };
                let item = &mut items[at];
                let sent = sent.min(item.size).max(item.sent);
                if item.status != UploadStatus::Uploading || sent == item.sent {
                    return Transition::stay(state.clone());
                }
                item.sent = sent;
                return Transition { state: UploadQueueState { items }, commands };
            }
            UploadQueueEvent::Complete(id) => {
                let Some(at) = find(&items, &id).filter(|&at| items[at].status == UploadStatus::Uploading) else {
                    return Transition::stay(state.clone());
                };
                items[at].status = UploadStatus::Done;
                items[at].sent = items[at].size;
                commands.push(UploadQueueCommand::Completed(id));
            }
            UploadQueueEvent::Fail { id, message } => {
                let Some(at) = find(&items, &id).filter(|&at| items[at].status == UploadStatus::Uploading) else {
                    return Transition::stay(state.clone());
                };
                items[at].status = UploadStatus::Failed;
                items[at].error = Some(message.clone());
                commands.push(UploadQueueCommand::Failed { id, message });
            }
            UploadQueueEvent::Pause(id) => {
                let Some(at) = find(&items, &id) else { return Transition::stay(state.clone()) };
                match items[at].status {
                    UploadStatus::Uploading => commands.push(UploadQueueCommand::Abort(id)),
                    UploadStatus::Queued => {}
                    _ => return Transition::stay(state.clone()),
                }
                items[at].status = UploadStatus::Paused;
            }
            UploadQueueEvent::Resume(id) => {
                let Some(at) = find(&items, &id).filter(|&at| items[at].status == UploadStatus::Paused) else {
                    return Transition::stay(state.clone());
                };
                items[at].status = UploadStatus::Queued;
            }
            UploadQueueEvent::Cancel(id) => {
                let Some(at) = find(&items, &id).filter(|&at| !items[at].status.is_settled()) else {
                    return Transition::stay(state.clone());
                };
                if items[at].status == UploadStatus::Uploading {
                    commands.push(UploadQueueCommand::Abort(id));
                }
                items[at].status = UploadStatus::Cancelled;
            }
            UploadQueueEvent::Retry(id) => {
                let Some(at) = find(&items, &id)
                    .filter(|&at| matches!(items[at].status, UploadStatus::Failed | UploadStatus::Cancelled))
                else {
                    return Transition::stay(state.clone());
                };
                items[at].status = UploadStatus::Queued;
                items[at].sent = 0;
                items[at].error = None;
            }
            UploadQueueEvent::Remove(id) => {
                let Some(at) = find(&items, &id) else { return Transition::stay(state.clone()) };
                if items[at].status == UploadStatus::Uploading {
                    commands.push(UploadQueueCommand::Abort(id.clone()));
                }
                items.remove(at);
                commands.push(UploadQueueCommand::Removed(id));
            }
            UploadQueueEvent::ClearFinished => {
                let (gone, kept): (Vec<_>, Vec<_>) =
                    items.into_iter().partition(|i| matches!(i.status, UploadStatus::Done | UploadStatus::Cancelled));
                if gone.is_empty() {
                    return Transition::stay(state.clone());
                }
                commands.extend(gone.into_iter().map(|i| UploadQueueCommand::Removed(i.id)));
                items = kept;
            }
        }
        schedule(&mut items, input, &mut commands);
        Transition { state: UploadQueueState { items }, commands }
    }

    fn sync(state: &UploadQueueState, _input: &UploadQueueInput) -> UploadQueueState {
        state.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(concurrency: usize) -> UploadQueueInput {
        UploadQueueInput { concurrency }
    }

    fn files(n: usize) -> Vec<UploadFile> {
        (0..n).map(|i| UploadFile::new(format!("f{i}"), format!("file{i}.txt"), 100)).collect()
    }

    fn run(s: &UploadQueueState, e: UploadQueueEvent, i: &UploadQueueInput) -> Step {
        UploadQueueMachine::transition(s, e, i)
    }

    fn status(s: &UploadQueueState) -> Vec<&'static str> {
        s.items.iter().map(|i| i.status.as_str()).collect()
    }

    fn start(id: &str, attempt: u32, from: u64) -> UploadQueueCommand {
        UploadQueueCommand::Start { id: id.into(), attempt, from }
    }

    #[test]
    fn add_starts_up_to_the_limit_and_the_rest_wait() {
        let i = input(2);
        let step = run(&UploadQueueMachine::initial(&i), UploadQueueEvent::Add(files(3)), &i);
        assert_eq!(status(&step.state), ["uploading", "uploading", "queued"]);
        assert_eq!(step.commands, [start("f0", 1, 0), start("f1", 1, 0)]);
        // A finished upload frees a slot for the next in line.
        let step = run(&step.state, UploadQueueEvent::Complete("f0".into()), &i);
        assert_eq!(status(&step.state), ["done", "uploading", "uploading"]);
        assert_eq!(step.commands, [UploadQueueCommand::Completed("f0".into()), start("f2", 1, 0)]);
        assert_eq!(step.state.items[0].percent(), 100);
    }

    #[test]
    fn duplicate_ids_are_ignored() {
        let i = input(1);
        let s = run(&UploadQueueMachine::initial(&i), UploadQueueEvent::Add(files(1)), &i).state;
        let step = run(&s, UploadQueueEvent::Add(files(1)), &i);
        assert_eq!(step.state.items.len(), 1);
        assert!(step.commands.is_empty());
    }

    #[test]
    fn progress_only_counts_while_uploading_and_never_goes_back() {
        let i = input(1);
        let s = run(&UploadQueueMachine::initial(&i), UploadQueueEvent::Add(files(2)), &i).state;
        let prog =
            |s: &UploadQueueState, id: &str, sent| run(s, UploadQueueEvent::Progress { id: id.into(), sent }, &i).state;
        let s = prog(&s, "f0", 40);
        assert_eq!(s.items[0].percent(), 40);
        assert_eq!(prog(&s, "f0", 10).items[0].sent, 40);
        assert_eq!(prog(&s, "f0", 999).items[0].sent, 100);
        // The second item isn't uploading yet.
        assert_eq!(prog(&s, "f1", 50).items[1].sent, 0);
        assert_eq!(prog(&s, "nope", 50), s);
    }

    #[test]
    fn pause_aborts_and_resume_continues_from_where_it_was() {
        let i = input(1);
        let s = run(&UploadQueueMachine::initial(&i), UploadQueueEvent::Add(files(2)), &i).state;
        let s = run(&s, UploadQueueEvent::Progress { id: "f0".into(), sent: 30 }, &i).state;
        let step = run(&s, UploadQueueEvent::Pause("f0".into()), &i);
        // The slot goes to the next item.
        assert_eq!(status(&step.state), ["paused", "uploading"]);
        assert_eq!(step.commands, [UploadQueueCommand::Abort("f0".into()), start("f1", 1, 0)]);
        let step = run(&step.state, UploadQueueEvent::Resume("f0".into()), &i);
        // Still full: it waits.
        assert_eq!(status(&step.state), ["queued", "uploading"]);
        let step = run(&step.state, UploadQueueEvent::Complete("f1".into()), &i);
        assert_eq!(step.commands.last(), Some(&start("f0", 2, 30)));
    }

    #[test]
    fn fail_then_retry_starts_over() {
        let i = input(1);
        let s = run(&UploadQueueMachine::initial(&i), UploadQueueEvent::Add(files(1)), &i).state;
        let s = run(&s, UploadQueueEvent::Progress { id: "f0".into(), sent: 60 }, &i).state;
        let step = run(&s, UploadQueueEvent::Fail { id: "f0".into(), message: "Network lost".into() }, &i);
        assert_eq!(step.state.items[0].status, UploadStatus::Failed);
        assert_eq!(step.state.items[0].error.as_deref(), Some("Network lost"));
        // A late progress report from the dead transfer is ignored.
        assert_eq!(run(&step.state, UploadQueueEvent::Progress { id: "f0".into(), sent: 80 }, &i).state, step.state);
        let step = run(&step.state, UploadQueueEvent::Retry("f0".into()), &i);
        assert_eq!(step.state.items[0].sent, 0);
        assert_eq!(step.commands, [start("f0", 2, 0)]);
    }

    #[test]
    fn cancel_remove_and_clear() {
        let i = input(1);
        let s = run(&UploadQueueMachine::initial(&i), UploadQueueEvent::Add(files(3)), &i).state;
        let step = run(&s, UploadQueueEvent::Cancel("f0".into()), &i);
        assert_eq!(status(&step.state), ["cancelled", "uploading", "queued"]);
        assert_eq!(step.commands, [UploadQueueCommand::Abort("f0".into()), start("f1", 1, 0)]);
        // Cancelled items can't be cancelled or paused again.
        assert!(run(&step.state, UploadQueueEvent::Cancel("f0".into()), &i).commands.is_empty());
        assert!(run(&step.state, UploadQueueEvent::Pause("f0".into()), &i).commands.is_empty());
        let step = run(&step.state, UploadQueueEvent::Remove("f1".into()), &i);
        assert_eq!(
            step.commands,
            [UploadQueueCommand::Abort("f1".into()), UploadQueueCommand::Removed("f1".into()), start("f2", 1, 0)]
        );
        let step = run(&step.state, UploadQueueEvent::ClearFinished, &i);
        assert_eq!(status(&step.state), ["uploading"]);
        assert_eq!(step.commands, [UploadQueueCommand::Removed("f0".into())]);
    }

    #[test]
    fn summary_and_percent() {
        let i = input(1);
        let s = run(&UploadQueueMachine::initial(&i), UploadQueueEvent::Add(files(3)), &i).state;
        let s = run(&s, UploadQueueEvent::Complete("f0".into()), &i).state;
        let sum = summarize(&s.items);
        assert_eq!((sum.total, sum.done, sum.uploading, sum.waiting), (3, 1, 1, 1));
        assert_eq!(UploadItem::new(UploadFile::new("e", "empty", 0)).percent(), 0);
    }
}
