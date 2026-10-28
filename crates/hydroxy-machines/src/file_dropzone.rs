//! File dropzone owns a list of picked files and the drag-over state.
//! `accept`, size and count limits are checked here, so rejections are pure
//! and tested. Each file carries an opaque handle `F` (a browser `File`, a
//! Dioxus `FileData`) next to its metadata, so adapters keep one list.

use std::marker::PhantomData;

use hydroxy_statechart::{Machine, Transition};

/// One picked file: metadata plus the platform handle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PickedFile<F> {
    /// File name, such as `"report.pdf"`.
    pub name: String,
    /// Size in bytes.
    pub size: u64,
    /// MIME type, such as `"application/pdf"`. May be empty.
    pub mime: String,
    /// The platform's file object.
    pub file: F,
}

/// Why a file was turned away.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RejectReason {
    /// Not matched by `accept`.
    FileType,
    /// Bigger than `max_size`.
    FileSize,
    /// Over `max_files`.
    FileCount,
}

impl RejectReason {
    /// A short sentence for the user.
    #[must_use]
    pub const fn message(self) -> &'static str {
        match self {
            RejectReason::FileType => "File type not allowed",
            RejectReason::FileSize => "File is too large",
            RejectReason::FileCount => "Too many files",
        }
    }
}

/// A file that was not added.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rejection<F> {
    /// The file.
    pub file: PickedFile<F>,
    /// Why.
    pub reason: RejectReason,
}

/// Props for [`FileDropzoneMachine`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileDropzoneInput {
    /// Comma-separated list like the native `accept`: `.pdf`, `image/*`, `text/plain`. Empty allows all.
    pub accept: String,
    /// Largest file in bytes.
    pub max_size: Option<u64>,
    /// Most files held at once. With `multiple` off this is always 1.
    pub max_files: Option<usize>,
    /// Hold many files. Off: a new pick replaces the current file.
    pub multiple: bool,
    /// Ignore drops, picks and removals.
    pub disabled: bool,
}

impl Default for FileDropzoneInput {
    fn default() -> Self {
        Self { accept: String::new(), max_size: None, max_files: None, multiple: true, disabled: false }
    }
}

/// Whether `accept` allows a file with this name and MIME type. Matches the
/// native `accept` rules: extensions, `type/*` wildcards and exact types.
#[must_use]
pub fn accepts(accept: &str, name: &str, mime: &str) -> bool {
    let name = name.to_ascii_lowercase();
    let mime = mime.to_ascii_lowercase();
    let mut tokens = accept.split(',').map(|t| t.trim().to_ascii_lowercase()).filter(|t| !t.is_empty()).peekable();
    if tokens.peek().is_none() {
        return true;
    }
    tokens.any(|t| {
        if t.starts_with('.') {
            name.ends_with(&t)
        } else if let Some(prefix) = t.strip_suffix('*') {
            mime.starts_with(prefix)
        } else {
            mime == t
        }
    })
}

/// What [`FileDropzoneMachine`] remembers between events.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileDropzoneState<F> {
    /// Accepted files, in pick order.
    pub files: Vec<PickedFile<F>>,
    /// Nested `dragenter` count; the zone is drag-over while above zero.
    pub drag_depth: u32,
}

impl<F> FileDropzoneState<F> {
    /// Files are being dragged over the zone.
    #[must_use]
    pub fn dragging(&self) -> bool {
        self.drag_depth > 0
    }
}

/// Inputs to [`FileDropzoneMachine`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileDropzoneEvent<F> {
    /// `dragenter` on the zone or a child.
    DragEnter,
    /// `dragleave` from the zone or a child.
    DragLeave,
    /// Files dropped or picked through the native input.
    Add(Vec<PickedFile<F>>),
    /// Remove the file at this index.
    Remove(usize),
    /// Remove every file.
    Clear,
}

/// Side effects from [`FileDropzoneMachine`] for the adapter to run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileDropzoneCommand<F> {
    /// The file list changed. Carries the whole new list.
    FilesChange(Vec<PickedFile<F>>),
    /// Some files were turned away.
    Reject(Vec<Rejection<F>>),
}

/// A drop area plus file list, generic over the platform file handle.
#[derive(Debug, Clone, Copy)]
pub struct FileDropzoneMachine<F>(PhantomData<F>);

impl<F: Clone> Machine for FileDropzoneMachine<F> {
    type Input = FileDropzoneInput;
    type State = FileDropzoneState<F>;
    type Event = FileDropzoneEvent<F>;
    type Command = FileDropzoneCommand<F>;

    fn initial(_input: &FileDropzoneInput) -> FileDropzoneState<F> {
        FileDropzoneState { files: Vec::new(), drag_depth: 0 }
    }

    fn transition(
        state: &FileDropzoneState<F>,
        event: FileDropzoneEvent<F>,
        input: &FileDropzoneInput,
    ) -> Transition<FileDropzoneState<F>, FileDropzoneCommand<F>> {
        let mut next = state.clone();
        if input.disabled {
            next.drag_depth = 0;
            return Transition::to(next);
        }
        let mut commands = Vec::new();
        match event {
            FileDropzoneEvent::DragEnter => next.drag_depth += 1,
            FileDropzoneEvent::DragLeave => next.drag_depth = next.drag_depth.saturating_sub(1),
            FileDropzoneEvent::Add(incoming) => {
                next.drag_depth = 0;
                let limit = if input.multiple { input.max_files.unwrap_or(usize::MAX) } else { 1 };
                let mut kept = if input.multiple { next.files.clone() } else { Vec::new() };
                let before = kept.len();
                let mut rejected = Vec::new();
                for file in incoming {
                    let reason = if !accepts(&input.accept, &file.name, &file.mime) {
                        Some(RejectReason::FileType)
                    } else if input.max_size.is_some_and(|max| file.size > max) {
                        Some(RejectReason::FileSize)
                    } else if kept.len() >= limit {
                        Some(RejectReason::FileCount)
                    } else {
                        None
                    };
                    match reason {
                        Some(reason) => rejected.push(Rejection { file, reason }),
                        None => kept.push(file),
                    }
                }
                if kept.len() > before {
                    next.files = kept;
                    commands.push(FileDropzoneCommand::FilesChange(next.files.clone()));
                }
                if !rejected.is_empty() {
                    commands.push(FileDropzoneCommand::Reject(rejected));
                }
            }
            FileDropzoneEvent::Remove(index) => {
                if index < next.files.len() {
                    next.files.remove(index);
                    commands.push(FileDropzoneCommand::FilesChange(next.files.clone()));
                }
            }
            FileDropzoneEvent::Clear => {
                if !next.files.is_empty() {
                    next.files.clear();
                    commands.push(FileDropzoneCommand::FilesChange(Vec::new()));
                }
            }
        }
        Transition::with(next, commands)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type M = FileDropzoneMachine<()>;

    fn file(name: &str, size: u64, mime: &str) -> PickedFile<()> {
        PickedFile { name: name.into(), size, mime: mime.into(), file: () }
    }

    fn add(
        state: &FileDropzoneState<()>,
        files: Vec<PickedFile<()>>,
        input: &FileDropzoneInput,
    ) -> hydroxy_statechart::Step<FileDropzoneState<()>, FileDropzoneCommand<()>> {
        M::transition(state, FileDropzoneEvent::Add(files), input).resolve(state)
    }

    #[test]
    fn accept_matches_extensions_wildcards_and_types() {
        assert!(accepts("", "a.bin", ""));
        assert!(accepts(".PDF, image/*", "Report.pdf", "application/pdf"));
        assert!(accepts("image/*", "a.png", "image/png"));
        assert!(accepts("text/plain", "a", "text/plain"));
        assert!(!accepts(".pdf,image/*", "a.txt", "text/plain"));
    }

    #[test]
    fn add_validates_type_size_and_count() {
        let input = FileDropzoneInput {
            accept: "image/*".into(),
            max_size: Some(100),
            max_files: Some(2),
            ..FileDropzoneInput::default()
        };
        let step = add(
            &M::initial(&input),
            vec![
                file("a.png", 10, "image/png"),
                file("b.txt", 10, "text/plain"),
                file("c.png", 500, "image/png"),
                file("d.png", 10, "image/png"),
                file("e.png", 10, "image/png"),
            ],
            &input,
        );
        let names: Vec<_> = step.state.files.iter().map(|f| f.name.as_str()).collect();
        assert_eq!(names, ["a.png", "d.png"]);
        let FileDropzoneCommand::Reject(rejected) = &step.commands[1] else { panic!("expected rejections") };
        let reasons: Vec<_> = rejected.iter().map(|r| (r.file.name.as_str(), r.reason)).collect();
        assert_eq!(
            reasons,
            [("b.txt", RejectReason::FileType), ("c.png", RejectReason::FileSize), ("e.png", RejectReason::FileCount)]
        );
    }

    #[test]
    fn count_limit_includes_files_already_held() {
        let input = FileDropzoneInput { max_files: Some(2), ..FileDropzoneInput::default() };
        let step = add(&M::initial(&input), vec![file("a", 1, "")], &input);
        let step = add(&step.state, vec![file("b", 1, ""), file("c", 1, "")], &input);
        assert_eq!(step.state.files.len(), 2);
        assert!(matches!(&step.commands[1], FileDropzoneCommand::Reject(r) if r[0].reason == RejectReason::FileCount));
    }

    #[test]
    fn single_mode_replaces() {
        let input = FileDropzoneInput { multiple: false, ..FileDropzoneInput::default() };
        let step = add(&M::initial(&input), vec![file("a", 1, "")], &input);
        let step = add(&step.state, vec![file("b", 1, ""), file("c", 1, "")], &input);
        assert_eq!(step.state.files, vec![file("b", 1, "")]);
        assert!(matches!(&step.commands[1], FileDropzoneCommand::Reject(r) if r.len() == 1));
    }

    #[test]
    fn all_rejected_keeps_files_and_reports_only_rejections() {
        let input = FileDropzoneInput { accept: ".pdf".into(), ..FileDropzoneInput::default() };
        let step = add(&M::initial(&input), vec![file("a.png", 1, "image/png")], &input);
        assert!(step.state.files.is_empty());
        assert!(matches!(step.commands.as_slice(), [FileDropzoneCommand::Reject(_)]));
    }

    #[test]
    fn remove_and_clear() {
        let input = FileDropzoneInput::default();
        let step = add(&M::initial(&input), vec![file("a", 1, ""), file("b", 1, "")], &input);
        let step = M::transition(&step.state, FileDropzoneEvent::Remove(0), &input).resolve(&step.state);
        assert_eq!(step.state.files, vec![file("b", 1, "")]);
        assert!(M::transition(&step.state, FileDropzoneEvent::Remove(9), &input).resolve(&step.state).commands.is_empty());
        let step = M::transition(&step.state, FileDropzoneEvent::Clear, &input).resolve(&step.state);
        assert!(step.state.files.is_empty());
        assert_eq!(step.commands, vec![FileDropzoneCommand::FilesChange(vec![])]);
    }

    #[test]
    fn drag_depth_survives_child_boundaries() {
        let input = FileDropzoneInput::default();
        let mut state = M::initial(&input);
        for event in [FileDropzoneEvent::DragEnter, FileDropzoneEvent::DragEnter, FileDropzoneEvent::DragLeave] {
            state = M::transition(&state, event, &input).resolve(&state).state;
        }
        assert!(state.dragging());
        state = add(&state, vec![], &input).state;
        assert!(!state.dragging());
        assert!(!M::transition(&state, FileDropzoneEvent::DragLeave, &input).resolve(&state).state.dragging());
    }

    #[test]
    fn disabled_ignores_everything() {
        let input = FileDropzoneInput { disabled: true, ..FileDropzoneInput::default() };
        let step = add(&M::initial(&input), vec![file("a", 1, "")], &input);
        assert!(step.state.files.is_empty());
        assert!(step.commands.is_empty());
    }
}
