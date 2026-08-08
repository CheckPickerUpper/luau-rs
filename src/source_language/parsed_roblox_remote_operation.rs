use crate::{source_language::ParsedExpression, SourceRange};

/// Names the explicit source operation used to cross a Roblox remote boundary.
#[derive(Clone, Copy)]
pub enum ParsedRobloxRemoteOperationKind {
    /// Connects a typed callback to the event direction owned by the module.
    Connect,
    /// Disconnects one previously returned connection.
    Disconnect,
    /// Sends one validated payload from a client to the server.
    FireServer,
    /// Sends one validated payload from the server to one client.
    FireClient,
    /// Sends one validated payload from the server to every client.
    FireAllClients,
    /// Invokes a server-owned remote function from a client.
    InvokeServer,
    /// Invokes a client-owned remote function from the server.
    InvokeClient,
    /// Installs a typed callback on a remote function in the module's direction.
    SetCallback,
}

/// Keeps remote operation syntax, its optional class argument, and source range together.
pub struct ParsedRobloxRemoteOperation {
    operation_kind: ParsedRobloxRemoteOperationKind,
    remote_type: Option<(String, SourceRange)>,
    arguments: Vec<ParsedExpression>,
    expression_range: SourceRange,
}

/// Preserves the complete operation until semantic checking can enforce its direction and shape.
impl ParsedRobloxRemoteOperation {
    /// Builds one parsed remote operation from its token-preserving parts.
    pub(crate) fn from_parts(
        operation_parts: (
            ParsedRobloxRemoteOperationKind,
            Option<(String, SourceRange)>,
            Vec<ParsedExpression>,
            SourceRange,
        ),
    ) -> Self {
        let (operation_kind, remote_type, arguments, expression_range) = operation_parts;
        Self {
            operation_kind,
            remote_type,
            arguments,
            expression_range,
        }
    }

    /// Gives semantic checking the requested remote operation.
    pub(crate) const fn operation_kind(&self) -> ParsedRobloxRemoteOperationKind {
        self.operation_kind
    }

    /// Gives semantic checking the optional generic remote class and its source range.
    pub(crate) fn remote_type(&self) -> Option<(&str, SourceRange)> {
        self.remote_type
            .as_ref()
            .map(|(remote_type_name, remote_type_range)| {
                (remote_type_name.as_str(), *remote_type_range)
            })
    }

    /// Gives semantic checking the ordered source arguments.
    pub(crate) fn arguments(&self) -> &[ParsedExpression] {
        &self.arguments
    }

    /// Gives diagnostics the complete source range of the remote operation.
    pub(crate) const fn expression_range(&self) -> SourceRange {
        self.expression_range
    }
}
