//! Assistant MCP: a manager of tool servers, one server row, the add-server form, and an
//! elicitation card where a server asks the person for input.
//!
//! The owner does the connecting. A row asks through `on_connect` and the owner reports back
//! through `outcome`; the row's machine decides which transitions are allowed.

mod add_form;
mod elicitation;
mod manager;
mod server;

pub use hydroxy_headless::assistant_mcp::{
    Connection, ElicitField, ElicitKind, ElicitStatus, FormStatus, McpServerInfo,
};

pub use add_form::*;
pub use elicitation::*;
pub use manager::*;
pub use server::*;
