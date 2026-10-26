//! Transcript viewer: which segment is playing, search, and whether the view follows playback.
//!
//! Following is the machine's one piece of state. It suspends when the reader
//! scrolls by hand and resumes on request, so the view never fights the reader.

use oxide_statechart::{Machine, Transition};

/// One spoken segment.
#[derive(Debug, Clone, PartialEq)]
pub struct Segment {
    /// Stable id.
    pub id: String,
    /// Who spoke.
    pub speaker: String,
    /// Start, seconds.
    pub start: f64,
    /// End, seconds.
    pub end: f64,
    /// What was said.
    pub text: String,
}

impl Segment {
    /// A segment.
    #[must_use]
    pub fn new(id: impl Into<String>, speaker: impl Into<String>, start: f64, end: f64, text: impl Into<String>) -> Self {
        Self { id: id.into(), speaker: speaker.into(), start, end, text: text.into() }
    }
}

/// Something odd about a segment's timing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Issue {
    /// The start is not a finite time.
    MissingStart,
    /// The end is not a finite time.
    MissingEnd,
    /// The end is not after the start.
    InvalidRange,
    /// It starts before the previous segment ends.
    Overlap,
}

fn finite(t: f64) -> bool {
    t.is_finite() && t >= 0.0
}

/// Timing problems per segment id. Segments with none are left out.
#[must_use]
pub fn diagnose(segments: &[Segment]) -> Vec<(String, Vec<Issue>)> {
    let mut out = Vec::new();
    let mut previous: Option<&Segment> = None;
    for s in segments {
        let mut issues = Vec::new();
        if !finite(s.start) {
            issues.push(Issue::MissingStart);
        }
        if !finite(s.end) {
            issues.push(Issue::MissingEnd);
        }
        if finite(s.start) && finite(s.end) && s.end <= s.start {
            issues.push(Issue::InvalidRange);
        }
        if let Some(p) = previous
            && finite(s.start)
            && s.start < p.end
        {
            issues.push(Issue::Overlap);
        }
        if !issues.is_empty() {
            out.push((s.id.clone(), issues));
        }
        if finite(s.start) && finite(s.end) {
            previous = Some(s);
        }
    }
    out
}

/// The last segment whose range contains `time`.
#[must_use]
pub fn active_segment(segments: &[Segment], time: f64) -> Option<&str> {
    if !time.is_finite() {
        return None;
    }
    segments
        .iter()
        .rfind(|s| finite(s.start) && s.start <= time && (!finite(s.end) || time < s.end))
        .map(|s| s.id.as_str())
}

/// Segments whose speaker or text contains the query, ignoring case.
#[must_use]
pub fn filter<'a>(segments: &'a [Segment], query: &str) -> Vec<&'a Segment> {
    let q = query.trim().to_lowercase();
    segments
        .iter()
        .filter(|s| q.is_empty() || format!("{} {}", s.speaker, s.text).to_lowercase().contains(&q))
        .collect()
}

/// Where a click on this segment should seek to.
#[must_use]
pub fn seek_time(segment: &Segment) -> f64 {
    if finite(segment.start) { segment.start } else { 0.0 }
}

/// `mm:ss`, or `--:--` for an unusable time.
#[must_use]
pub fn format_time(seconds: f64) -> String {
    if !finite(seconds) {
        return "--:--".to_string();
    }
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss, reason = "non-negative and far below u64::MAX")]
    let total = seconds as u64;
    format!("{:02}:{:02}", total / 60, total % 60)
}

/// Whether the view scrolls to keep the playing segment visible.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TranscriptViewerState {
    /// Following playback.
    pub following: bool,
}

impl Default for TranscriptViewerState {
    fn default() -> Self {
        Self { following: true }
    }
}

/// Props for [`TranscriptViewerMachine`].
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TranscriptViewerInput {
    /// Start suspended.
    pub start_suspended: bool,
}

/// Inputs to [`TranscriptViewerMachine`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TranscriptViewerEvent {
    /// The reader scrolled by wheel, touch or key.
    UserScroll,
    /// The reader asked to follow again.
    Resume,
}

/// Side effects from [`TranscriptViewerMachine`] for the adapter to run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TranscriptViewerCommand {
    /// Following changed; report it.
    FollowChange(bool),
    /// Scroll the playing segment into view now.
    ScrollToActive,
}

/// Following or suspended.
#[derive(Debug, Clone, Copy)]
pub struct TranscriptViewerMachine;

impl Machine for TranscriptViewerMachine {
    type Input = TranscriptViewerInput;
    type State = TranscriptViewerState;
    type Event = TranscriptViewerEvent;
    type Command = TranscriptViewerCommand;

    fn initial(input: &TranscriptViewerInput) -> TranscriptViewerState {
        TranscriptViewerState { following: !input.start_suspended }
    }

    fn transition(
        state: &TranscriptViewerState,
        event: TranscriptViewerEvent,
        _input: &TranscriptViewerInput,
    ) -> Transition<TranscriptViewerState, TranscriptViewerCommand> {
        let (following, commands) = match (event, state.following) {
            (TranscriptViewerEvent::UserScroll, true) => (false, vec![TranscriptViewerCommand::FollowChange(false)]),
            (TranscriptViewerEvent::Resume, false) => {
                (true, vec![TranscriptViewerCommand::FollowChange(true), TranscriptViewerCommand::ScrollToActive])
            }
            _ => (state.following, Vec::new()),
        };
        Transition { state: TranscriptViewerState { following }, commands }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn segments() -> Vec<Segment> {
        vec![
            Segment::new("a", "Ana", 0.0, 4.0, "Welcome to the show."),
            Segment::new("b", "Ben", 4.0, 9.0, "Thanks for having me."),
            Segment::new("c", "Ana", 8.0, 12.0, "Let's talk about audio."),
        ]
    }

    #[test]
    fn active_is_the_last_containing_segment() {
        let s = segments();
        assert_eq!(active_segment(&s, 0.0), Some("a"));
        assert_eq!(active_segment(&s, 4.0), Some("b"));
        assert_eq!(active_segment(&s, 8.5), Some("c"));
        assert_eq!(active_segment(&s, 12.0), None);
        assert_eq!(active_segment(&s, f64::NAN), None);
    }

    #[test]
    fn diagnoses_overlap_and_bad_ranges() {
        let mut s = segments();
        s.push(Segment::new("d", "Ben", 20.0, 20.0, "x"));
        let d = diagnose(&s);
        assert_eq!(d, [("c".to_string(), vec![Issue::Overlap]), ("d".to_string(), vec![Issue::InvalidRange])]);
    }

    #[test]
    fn search_matches_speaker_and_text() {
        let s = segments();
        assert_eq!(filter(&s, "ben").len(), 1);
        assert_eq!(filter(&s, " AUDIO ").len(), 1);
        assert_eq!(filter(&s, "").len(), 3);
    }

    #[test]
    fn formats_time() {
        assert_eq!(format_time(65.9), "01:05");
        assert_eq!(format_time(f64::NAN), "--:--");
    }

    #[test]
    fn scrolling_suspends_and_resume_follows() {
        let input = TranscriptViewerInput::default();
        let s = TranscriptViewerMachine::initial(&input);
        let t = TranscriptViewerMachine::transition(&s, TranscriptViewerEvent::UserScroll, &input);
        assert!(!t.state.following);
        assert_eq!(t.commands, [TranscriptViewerCommand::FollowChange(false)]);
        assert!(TranscriptViewerMachine::transition(&t.state, TranscriptViewerEvent::UserScroll, &input).commands.is_empty());
        let r = TranscriptViewerMachine::transition(&t.state, TranscriptViewerEvent::Resume, &input);
        assert!(r.state.following);
        assert_eq!(r.commands, [TranscriptViewerCommand::FollowChange(true), TranscriptViewerCommand::ScrollToActive]);
    }
}
