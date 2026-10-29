//! Assistant MCP connector. Scopes and parts:
//! - `assistant-mcp-manager`: root, connectors, custom-servers, add-trigger
//! - `assistant-mcp-server`: root, icon, name, status, tools, connect-button,
//!   disconnect-button, remove-button
//! - `assistant-mcp-add-form`: root, input, error, submit
//! - `assistant-mcp-elicitation`: root, form, field, submit, cancel

pub use hydroxy_machines::assistant_mcp::*;

/// `data-scope` of the manager.
pub const SCOPE_MANAGER: &str = "assistant-mcp-manager";
/// `data-scope` of a server row.
pub const SCOPE_SERVER: &str = "assistant-mcp-server";
/// `data-scope` of the add form.
pub const SCOPE_ADD_FORM: &str = "assistant-mcp-add-form";
/// `data-scope` of an elicitation.
pub const SCOPE_ELICITATION: &str = "assistant-mcp-elicitation";

/// The error under a required elicitation field left empty.
pub const REQUIRED_TEXT: &str = "This answer is required.";

/// Id of an elicitation field's error, for `aria-describedby`. `uid` is per card, so two
/// cards asking for the same field don't share ids.
#[must_use]
pub fn field_error_id(uid: &str, field: &str) -> String {
    format!("{uid}-{field}-error")
}

/// A server in the manager. The owner holds the list and each server's connection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct McpServerInfo {
    /// Stable key.
    pub id: String,
    /// Display name.
    pub name: String,
    /// What it offers.
    pub description: String,
    /// Added by the person, so it can be removed.
    pub custom: bool,
}

impl McpServerInfo {
    /// A built-in connector.
    pub fn connector(id: impl Into<String>, name: impl Into<String>, description: impl Into<String>) -> Self {
        Self { id: id.into(), name: name.into(), description: description.into(), custom: false }
    }

    /// A server the person added.
    pub fn custom(id: impl Into<String>, name: impl Into<String>, description: impl Into<String>) -> Self {
        Self { id: id.into(), name: name.into(), description: description.into(), custom: true }
    }

    /// First letter of the name, for the icon.
    #[must_use]
    pub fn initial(&self) -> String {
        self.name.chars().next().map(|c| c.to_uppercase().collect()).unwrap_or_default()
    }
}

/// The status text for a connection.
#[must_use]
pub fn status_label(connection: &Connection, tools: usize) -> String {
    match connection {
        Connection::Disconnected => "Not connected".to_string(),
        Connection::Connecting => "Connecting".to_string(),
        Connection::Connected => format!("Connected, {tools} {}", if tools == 1 { "tool" } else { "tools" }),
        Connection::Error(msg) => format!("Failed: {msg}"),
    }
}

/// Connect is offered when not connected or after a failure.
#[must_use]
pub fn can_connect(connection: &Connection) -> bool {
    matches!(connection, Connection::Disconnected | Connection::Error(_))
}

/// Disconnect is offered while connecting or connected.
#[must_use]
pub fn can_disconnect(connection: &Connection) -> bool {
    matches!(connection, Connection::Connecting | Connection::Connected)
}

/// Split servers into (connectors, custom), keeping order.
#[must_use]
pub fn split_servers(servers: &[McpServerInfo]) -> (Vec<&McpServerInfo>, Vec<&McpServerInfo>) {
    servers.iter().partition(|s| !s.custom)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_text_per_state() {
        assert_eq!(status_label(&Connection::Disconnected, 0), "Not connected");
        assert_eq!(status_label(&Connection::Connected, 1), "Connected, 1 tool");
        assert_eq!(status_label(&Connection::Connected, 3), "Connected, 3 tools");
        assert_eq!(status_label(&Connection::Error("timeout".into()), 0), "Failed: timeout");
    }

    #[test]
    fn buttons_follow_the_connection() {
        assert!(can_connect(&Connection::Error("x".into())));
        assert!(!can_connect(&Connection::Connected));
        assert!(can_disconnect(&Connection::Connecting));
        assert!(!can_disconnect(&Connection::Disconnected));
    }

    #[test]
    fn servers_split_and_get_an_initial() {
        let s = vec![McpServerInfo::connector("a", "github", ""), McpServerInfo::custom("b", "Mine", "")];
        let (c, u) = split_servers(&s);
        assert_eq!((c.len(), u.len()), (1, 1));
        assert_eq!(s[0].initial(), "G");
    }
}
