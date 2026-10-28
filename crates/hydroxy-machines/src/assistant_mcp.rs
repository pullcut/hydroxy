//! MCP server management for an assistant: one server's connection, the add-server form,
//! and an elicitation (a server asking the person for input). The owner holds the server
//! list and does the connecting; these machines hold the state around it.

use hydroxy_statechart::{Machine, Transition};

// --------------------------------------------------------------------- server

/// Where a server's connection stands.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum Connection {
    /// Not connected.
    #[default]
    Disconnected,
    /// Connecting.
    Connecting,
    /// Connected.
    Connected,
    /// The last attempt failed.
    Error(String),
}

impl Connection {
    /// The `data-connection-state` value.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Connection::Disconnected => "disconnected",
            Connection::Connecting => "connecting",
            Connection::Connected => "connected",
            Connection::Error(_) => "error",
        }
    }
}

/// What [`McpServerMachine`] remembers.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct McpServerState {
    /// The connection.
    pub connection: Connection,
    /// Tool names the server offers once connected.
    pub tools: Vec<String>,
}

/// Inputs to [`McpServerMachine`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum McpServerEvent {
    /// The person pressed Connect (or Retry).
    Connect,
    /// The owner's connection succeeded.
    Connected(Vec<String>),
    /// The owner's connection failed.
    Failed(String),
    /// The person pressed Disconnect.
    Disconnect,
}

/// Side effects for the adapter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum McpServerCommand {
    /// Open the connection, then send `Connected` or `Failed`.
    Connect,
    /// Close the connection.
    Disconnect,
}

/// One MCP server's connection.
#[derive(Debug, Clone, Copy)]
pub struct McpServerMachine;

impl Machine for McpServerMachine {
    type Input = ();
    type State = McpServerState;
    type Event = McpServerEvent;
    type Command = McpServerCommand;

    fn initial((): &()) -> McpServerState {
        McpServerState::default()
    }

    fn transition(
        state: &McpServerState,
        event: McpServerEvent,
        (): &(),
    ) -> Transition<McpServerState, McpServerCommand> {
        let mut next = state.clone();
        let mut commands = Vec::new();
        match (&state.connection, event) {
            (Connection::Disconnected | Connection::Error(_), McpServerEvent::Connect) => {
                next.connection = Connection::Connecting;
                commands.push(McpServerCommand::Connect);
            }
            (Connection::Connecting, McpServerEvent::Connected(tools)) => {
                next.connection = Connection::Connected;
                next.tools = tools;
            }
            (Connection::Connecting, McpServerEvent::Failed(msg)) => {
                next.connection = Connection::Error(msg);
                next.tools.clear();
            }
            (Connection::Connected | Connection::Connecting, McpServerEvent::Disconnect) => {
                next.connection = Connection::Disconnected;
                next.tools.clear();
                commands.push(McpServerCommand::Disconnect);
            }
            _ => {}
        }
        Transition::with(next, commands)
    }
}

// ------------------------------------------------------------------- add form

/// Where [`McpAddFormMachine`] stands.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum FormStatus {
    /// Editing.
    #[default]
    Idle,
    /// Waiting for the owner.
    Submitting,
    /// The owner accepted the values.
    Submitted,
    /// The owner rejected the values.
    Error,
}

/// What [`McpAddFormMachine`] remembers.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct McpAddFormState {
    /// Display name.
    pub name: String,
    /// Server URL.
    pub url: String,
    /// Where the form stands.
    pub status: FormStatus,
    /// Problem with the name, after a failed submit.
    pub name_error: Option<String>,
    /// Problem with the URL, after a failed submit.
    pub url_error: Option<String>,
    /// Problem reported by the owner.
    pub error: Option<String>,
}

/// A field of the add form.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum McpField {
    /// The display name.
    Name,
    /// The server URL.
    Url,
}

/// Inputs to [`McpAddFormMachine`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum McpAddFormEvent {
    /// Edit a field.
    Set(McpField, String),
    /// Submit.
    Submit,
    /// The owner accepted the server.
    Succeeded,
    /// The owner rejected the server.
    Failed(String),
    /// Clear the form.
    Reset,
}

/// Side effects for the adapter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum McpAddFormCommand {
    /// Add this server, then send `Succeeded` or `Failed`.
    Submit {
        /// Trimmed name.
        name: String,
        /// Trimmed URL.
        url: String,
    },
}

/// Check a server URL: it needs an http or https scheme and a host.
#[must_use]
pub fn validate_url(url: &str) -> Option<&'static str> {
    let url = url.trim();
    if url.is_empty() {
        return Some("Enter the server URL.");
    }
    let host = url.strip_prefix("https://").or_else(|| url.strip_prefix("http://"));
    match host {
        Some(h) if !h.is_empty() && !h.starts_with('/') && !h.contains(char::is_whitespace) => None,
        _ => Some("Use a full http:// or https:// address."),
    }
}

/// The add-server form.
#[derive(Debug, Clone, Copy)]
pub struct McpAddFormMachine;

impl Machine for McpAddFormMachine {
    type Input = ();
    type State = McpAddFormState;
    type Event = McpAddFormEvent;
    type Command = McpAddFormCommand;

    fn initial((): &()) -> McpAddFormState {
        McpAddFormState::default()
    }

    fn transition(
        state: &McpAddFormState,
        event: McpAddFormEvent,
        (): &(),
    ) -> Transition<McpAddFormState, McpAddFormCommand> {
        let mut next = state.clone();
        let mut commands = Vec::new();
        match event {
            McpAddFormEvent::Set(field, value) => {
                match field {
                    McpField::Name => {
                        next.name = value;
                        next.name_error = None;
                    }
                    McpField::Url => {
                        next.url = value;
                        next.url_error = None;
                    }
                }
                next.status = FormStatus::Idle;
                next.error = None;
            }
            McpAddFormEvent::Submit => {
                if state.status != FormStatus::Submitting {
                    next.name_error = state.name.trim().is_empty().then(|| "Enter a name.".to_string());
                    next.url_error = validate_url(&state.url).map(str::to_string);
                    if next.name_error.is_none() && next.url_error.is_none() {
                        next.status = FormStatus::Submitting;
                        next.error = None;
                        commands.push(McpAddFormCommand::Submit {
                            name: state.name.trim().to_string(),
                            url: state.url.trim().to_string(),
                        });
                    }
                }
            }
            McpAddFormEvent::Succeeded => {
                if state.status == FormStatus::Submitting {
                    next = McpAddFormState { status: FormStatus::Submitted, ..McpAddFormState::default() };
                }
            }
            McpAddFormEvent::Failed(msg) => {
                if state.status == FormStatus::Submitting {
                    next.status = FormStatus::Error;
                    next.error = Some(msg);
                }
            }
            McpAddFormEvent::Reset => next = McpAddFormState::default(),
        }
        Transition::with(next, commands)
    }
}

// ---------------------------------------------------------------- elicitation

/// What kind of answer a field takes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ElicitKind {
    /// Free text.
    Text,
    /// Yes or no. The value is `"true"` or `"false"`.
    Bool,
    /// One of the listed options.
    Choice(Vec<String>),
}

/// One question a server asks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ElicitField {
    /// Key in the answer.
    pub name: String,
    /// Label shown to the person.
    pub label: String,
    /// Answer type.
    pub kind: ElicitKind,
    /// The server needs an answer.
    pub required: bool,
}

impl ElicitField {
    /// A required text field.
    pub fn text(name: impl Into<String>, label: impl Into<String>) -> Self {
        Self { name: name.into(), label: label.into(), kind: ElicitKind::Text, required: true }
    }

    /// A yes/no field.
    pub fn bool(name: impl Into<String>, label: impl Into<String>) -> Self {
        Self { name: name.into(), label: label.into(), kind: ElicitKind::Bool, required: false }
    }

    /// A single choice.
    pub fn choice(name: impl Into<String>, label: impl Into<String>, options: &[&str]) -> Self {
        Self {
            name: name.into(),
            label: label.into(),
            kind: ElicitKind::Choice(options.iter().map(|o| (*o).to_string()).collect()),
            required: true,
        }
    }

    /// Make the answer optional.
    #[must_use]
    pub fn optional(mut self) -> Self {
        self.required = false;
        self
    }
}

/// Props for [`McpElicitationMachine`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct McpElicitationInput {
    /// The questions.
    pub fields: Vec<ElicitField>,
}

/// Where an elicitation stands.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ElicitStatus {
    /// Waiting for the person.
    #[default]
    Pending,
    /// Answered.
    Submitted,
    /// Declined.
    Cancelled,
}

impl ElicitStatus {
    /// The `data-status` value.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            ElicitStatus::Pending => "pending",
            ElicitStatus::Submitted => "submitted",
            ElicitStatus::Cancelled => "cancelled",
        }
    }
}

/// What [`McpElicitationMachine`] remembers.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct McpElicitationState {
    /// Answers so far, by field name.
    pub values: Vec<(String, String)>,
    /// Fields that failed the last submit.
    pub missing: Vec<String>,
    /// Where it stands.
    pub status: ElicitStatus,
}

impl McpElicitationState {
    /// The answer for `name`, or `""`.
    #[must_use]
    pub fn value(&self, name: &str) -> &str {
        self.values.iter().find(|(n, _)| n == name).map_or("", |(_, v)| v.as_str())
    }
}

/// Inputs to [`McpElicitationMachine`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum McpElicitationEvent {
    /// Answer a field.
    Set {
        /// Field name.
        name: String,
        /// Answer.
        value: String,
    },
    /// Send the answers.
    Submit,
    /// Decline.
    Cancel,
}

/// Side effects for the adapter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum McpElicitationCommand {
    /// Send these answers to the server.
    Submit(Vec<(String, String)>),
    /// Tell the server the person declined.
    Cancel,
}

/// A server asking the person for input.
#[derive(Debug, Clone, Copy)]
pub struct McpElicitationMachine;

impl Machine for McpElicitationMachine {
    type Input = McpElicitationInput;
    type State = McpElicitationState;
    type Event = McpElicitationEvent;
    type Command = McpElicitationCommand;

    fn initial(input: &McpElicitationInput) -> McpElicitationState {
        // A yes/no field starts as "no" and a choice starts on its first option, so
        // what is shown is what gets sent.
        let values = input
            .fields
            .iter()
            .filter_map(|f| match &f.kind {
                ElicitKind::Bool => Some((f.name.clone(), "false".to_string())),
                ElicitKind::Choice(o) => o.first().map(|first| (f.name.clone(), first.clone())),
                ElicitKind::Text => None,
            })
            .collect();
        McpElicitationState { values, ..McpElicitationState::default() }
    }

    fn transition(
        state: &McpElicitationState,
        event: McpElicitationEvent,
        input: &McpElicitationInput,
    ) -> Transition<McpElicitationState, McpElicitationCommand> {
        let mut next = state.clone();
        let mut commands = Vec::new();
        if state.status != ElicitStatus::Pending {
            return Transition::with(next, commands);
        }
        match event {
            McpElicitationEvent::Set { name, value } => {
                next.missing.retain(|m| *m != name);
                match next.values.iter_mut().find(|(n, _)| *n == name) {
                    Some(slot) => slot.1 = value,
                    None => next.values.push((name, value)),
                }
            }
            McpElicitationEvent::Submit => {
                next.missing = input
                    .fields
                    .iter()
                    .filter(|f| f.required && state.value(&f.name).trim().is_empty())
                    .map(|f| f.name.clone())
                    .collect();
                if next.missing.is_empty() {
                    next.status = ElicitStatus::Submitted;
                    let answers = input
                        .fields
                        .iter()
                        .filter(|f| !state.value(&f.name).trim().is_empty())
                        .map(|f| (f.name.clone(), state.value(&f.name).trim().to_string()))
                        .collect();
                    commands.push(McpElicitationCommand::Submit(answers));
                }
            }
            McpElicitationEvent::Cancel => {
                next.status = ElicitStatus::Cancelled;
                commands.push(McpElicitationCommand::Cancel);
            }
        }
        Transition::with(next, commands)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn server(s: &McpServerState, e: McpServerEvent) -> hydroxy_statechart::Step<McpServerState, McpServerCommand> {
        McpServerMachine::transition(s, e, &()).resolve(s)
    }

    #[test]
    fn server_connects_then_lists_tools() {
        let t = server(&McpServerState::default(), McpServerEvent::Connect);
        assert_eq!(t.state.connection, Connection::Connecting);
        assert_eq!(t.commands, vec![McpServerCommand::Connect]);
        let t = server(&t.state, McpServerEvent::Connected(vec!["search".into()]));
        assert_eq!(t.state.connection, Connection::Connected);
        assert_eq!(t.state.tools, vec!["search"]);
        let t = server(&t.state, McpServerEvent::Disconnect);
        assert_eq!(t.state.connection, Connection::Disconnected);
        assert!(t.state.tools.is_empty());
        assert_eq!(t.commands, vec![McpServerCommand::Disconnect]);
    }

    #[test]
    fn server_failure_can_be_retried() {
        let s = server(&McpServerState::default(), McpServerEvent::Connect).state;
        let s = server(&s, McpServerEvent::Failed("refused".into())).state;
        assert_eq!(s.connection, Connection::Error("refused".into()));
        assert_eq!(server(&s, McpServerEvent::Connect).commands, vec![McpServerCommand::Connect]);
    }

    #[test]
    fn server_ignores_events_that_do_not_apply() {
        let s = McpServerState::default();
        assert!(server(&s, McpServerEvent::Disconnect).commands.is_empty());
        assert_eq!(server(&s, McpServerEvent::Connected(vec![])).state, s);
    }

    #[test]
    fn urls_need_a_scheme_and_host() {
        assert_eq!(validate_url("https://mcp.example.com/sse"), None);
        assert!(validate_url("").is_some());
        assert!(validate_url("example.com").is_some());
        assert!(validate_url("https://").is_some());
        assert!(validate_url("https://a b").is_some());
    }

    fn form(s: &McpAddFormState, e: McpAddFormEvent) -> hydroxy_statechart::Step<McpAddFormState, McpAddFormCommand> {
        McpAddFormMachine::transition(s, e, &()).resolve(s)
    }

    #[test]
    fn form_flags_both_fields_and_does_not_submit() {
        let t = form(&McpAddFormState::default(), McpAddFormEvent::Submit);
        assert!(t.commands.is_empty());
        assert!(t.state.name_error.is_some() && t.state.url_error.is_some());
        let t = form(&t.state, McpAddFormEvent::Set(McpField::Name, "Docs".into()));
        assert!(t.state.name_error.is_none() && t.state.url_error.is_some());
    }

    #[test]
    fn form_submits_trimmed_then_resets_on_success() {
        let s = form(&McpAddFormState::default(), McpAddFormEvent::Set(McpField::Name, " Docs ".into())).state;
        let s = form(&s, McpAddFormEvent::Set(McpField::Url, " https://docs.example.com ".into())).state;
        let t = form(&s, McpAddFormEvent::Submit);
        assert_eq!(
            t.commands,
            vec![McpAddFormCommand::Submit { name: "Docs".into(), url: "https://docs.example.com".into() }]
        );
        assert_eq!(t.state.status, FormStatus::Submitting);
        assert!(form(&t.state, McpAddFormEvent::Submit).commands.is_empty());
        let done = form(&t.state, McpAddFormEvent::Succeeded).state;
        assert_eq!(done.status, FormStatus::Submitted);
        assert!(done.name.is_empty());
    }

    #[test]
    fn form_keeps_values_when_the_owner_rejects() {
        let s = form(&McpAddFormState::default(), McpAddFormEvent::Set(McpField::Name, "Docs".into())).state;
        let s = form(&s, McpAddFormEvent::Set(McpField::Url, "https://docs.example.com".into())).state;
        let s = form(&s, McpAddFormEvent::Submit).state;
        let s = form(&s, McpAddFormEvent::Failed("Already added".into())).state;
        assert_eq!(s.status, FormStatus::Error);
        assert_eq!(s.error.as_deref(), Some("Already added"));
        assert_eq!(s.name, "Docs");
    }

    fn input() -> McpElicitationInput {
        McpElicitationInput {
            fields: vec![
                ElicitField::text("repo", "Repository"),
                ElicitField::bool("private", "Private"),
                ElicitField::choice("branch", "Branch", &["main", "dev"]),
                ElicitField::text("note", "Note").optional(),
            ],
        }
    }

    fn ask(
        s: &McpElicitationState,
        e: McpElicitationEvent,
    ) -> hydroxy_statechart::Step<McpElicitationState, McpElicitationCommand> {
        McpElicitationMachine::transition(s, e, &input()).resolve(s)
    }

    #[test]
    fn elicitation_starts_with_visible_defaults() {
        let s = McpElicitationMachine::initial(&input());
        assert_eq!(s.value("private"), "false");
        assert_eq!(s.value("branch"), "main");
        assert_eq!(s.value("repo"), "");
    }

    #[test]
    fn elicitation_names_missing_required_fields() {
        let s = McpElicitationMachine::initial(&input());
        let t = ask(&s, McpElicitationEvent::Submit);
        assert!(t.commands.is_empty());
        assert_eq!(t.state.missing, vec!["repo"]);
        let t = ask(&t.state, McpElicitationEvent::Set { name: "repo".into(), value: "hydroxy".into() });
        assert!(t.state.missing.is_empty());
    }

    #[test]
    fn elicitation_submits_answers_in_field_order_and_locks() {
        let s = McpElicitationMachine::initial(&input());
        let s = ask(&s, McpElicitationEvent::Set { name: "repo".into(), value: " hydroxy ".into() }).state;
        let t = ask(&s, McpElicitationEvent::Submit);
        assert_eq!(
            t.commands,
            vec![McpElicitationCommand::Submit(vec![
                ("repo".into(), "hydroxy".into()),
                ("private".into(), "false".into()),
                ("branch".into(), "main".into()),
            ])]
        );
        assert_eq!(t.state.status, ElicitStatus::Submitted);
        assert!(ask(&t.state, McpElicitationEvent::Cancel).commands.is_empty());
    }

    #[test]
    fn elicitation_can_be_declined() {
        let s = McpElicitationMachine::initial(&input());
        let t = ask(&s, McpElicitationEvent::Cancel);
        assert_eq!(t.commands, vec![McpElicitationCommand::Cancel]);
        assert_eq!(t.state.status, ElicitStatus::Cancelled);
    }
}
