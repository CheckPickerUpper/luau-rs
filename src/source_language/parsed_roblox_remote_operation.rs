use crate::{source_language::ParsedExpression, SourceRange};

/// Names the explicit source operation used to cross a Roblox remote boundary.
#[derive(Clone, Copy)]
pub enum ParsedRobloxRemoteOperationKind {
    /// Connects a typed callback to the event direction owned by the module.
    Connect,
    /// Disconnects one previously returned connection.
    Disconnect,
    /// Waits for one client-directed event payload.
    Wait,
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

type RemoteType = (String, SourceRange);

/// Stores only argument combinations accepted by each remote operation's grammar.
pub enum ParsedRobloxRemoteOperation {
    /// Connects a remote event to a callback.
    Connect {
        remote_type: RemoteType,
        remote_expression: Box<ParsedExpression>,
        callback_expression: Box<ParsedExpression>,
        expression_range: SourceRange,
    },
    /// Disconnects one checked connection.
    Disconnect {
        connection_expression: Box<ParsedExpression>,
        expression_range: SourceRange,
    },
    /// Waits for a client event payload at an explicit yielding boundary.
    Wait {
        remote_type: RemoteType,
        remote_expression: Box<ParsedExpression>,
        expression_range: SourceRange,
    },
    /// Sends one payload to the server.
    FireServer {
        remote_type: RemoteType,
        remote_expression: Box<ParsedExpression>,
        payload_expression: Box<ParsedExpression>,
        expression_range: SourceRange,
    },
    /// Sends one payload to one client.
    FireClient {
        remote_type: RemoteType,
        remote_expression: Box<ParsedExpression>,
        player_expression: Box<ParsedExpression>,
        payload_expression: Box<ParsedExpression>,
        expression_range: SourceRange,
    },
    /// Sends one payload to every client.
    FireAllClients {
        remote_type: RemoteType,
        remote_expression: Box<ParsedExpression>,
        payload_expression: Box<ParsedExpression>,
        expression_range: SourceRange,
    },
    /// Calls the server and receives a string result.
    InvokeServer {
        remote_type: RemoteType,
        remote_expression: Box<ParsedExpression>,
        payload_expression: Box<ParsedExpression>,
        expression_range: SourceRange,
    },
    /// Calls one client and receives a string result.
    InvokeClient {
        remote_type: RemoteType,
        remote_expression: Box<ParsedExpression>,
        player_expression: Box<ParsedExpression>,
        payload_expression: Box<ParsedExpression>,
        expression_range: SourceRange,
    },
    /// Installs a remote-function callback.
    SetCallback {
        remote_type: RemoteType,
        remote_expression: Box<ParsedExpression>,
        callback_expression: Box<ParsedExpression>,
        expression_range: SourceRange,
    },
}

/// Rejects malformed operation shapes before semantic checking receives them.
impl ParsedRobloxRemoteOperation {
    pub(crate) fn from_syntax(
        syntax_parts: (
            ParsedRobloxRemoteOperationKind,
            Option<RemoteType>,
            Vec<ParsedExpression>,
            SourceRange,
        ),
    ) -> Option<Self> {
        let (operation_kind, remote_type, mut arguments, expression_range) = syntax_parts;
        match operation_kind {
            ParsedRobloxRemoteOperationKind::Disconnect if arguments.len() == 1 => {
                let connection_expression = arguments.pop()?;
                Some(Self::Disconnect {
                    connection_expression: Box::new(connection_expression),
                    expression_range,
                })
            }
            ParsedRobloxRemoteOperationKind::Connect
            | ParsedRobloxRemoteOperationKind::FireServer
            | ParsedRobloxRemoteOperationKind::FireAllClients
            | ParsedRobloxRemoteOperationKind::InvokeServer
            | ParsedRobloxRemoteOperationKind::SetCallback
                if arguments.len() == 2 =>
            {
                let second_expression = arguments.pop()?;
                let remote_expression = arguments.pop()?;
                let remote_type = remote_type?;
                Some(match operation_kind {
                    ParsedRobloxRemoteOperationKind::Connect => Self::Connect {
                        remote_type,
                        remote_expression: Box::new(remote_expression),
                        callback_expression: Box::new(second_expression),
                        expression_range,
                    },
                    ParsedRobloxRemoteOperationKind::FireServer => Self::FireServer {
                        remote_type,
                        remote_expression: Box::new(remote_expression),
                        payload_expression: Box::new(second_expression),
                        expression_range,
                    },
                    ParsedRobloxRemoteOperationKind::FireAllClients => Self::FireAllClients {
                        remote_type,
                        remote_expression: Box::new(remote_expression),
                        payload_expression: Box::new(second_expression),
                        expression_range,
                    },
                    ParsedRobloxRemoteOperationKind::InvokeServer => Self::InvokeServer {
                        remote_type,
                        remote_expression: Box::new(remote_expression),
                        payload_expression: Box::new(second_expression),
                        expression_range,
                    },
                    ParsedRobloxRemoteOperationKind::SetCallback => Self::SetCallback {
                        remote_type,
                        remote_expression: Box::new(remote_expression),
                        callback_expression: Box::new(second_expression),
                        expression_range,
                    },
                    ParsedRobloxRemoteOperationKind::Disconnect
                    | ParsedRobloxRemoteOperationKind::Wait
                    | ParsedRobloxRemoteOperationKind::FireClient
                    | ParsedRobloxRemoteOperationKind::InvokeClient => return None,
                })
            }
            ParsedRobloxRemoteOperationKind::FireClient
            | ParsedRobloxRemoteOperationKind::InvokeClient
                if arguments.len() == 3 =>
            {
                let payload_expression = arguments.pop()?;
                let player_expression = arguments.pop()?;
                let remote_expression = arguments.pop()?;
                let remote_type = remote_type?;
                Some(match operation_kind {
                    ParsedRobloxRemoteOperationKind::FireClient => Self::FireClient {
                        remote_type,
                        remote_expression: Box::new(remote_expression),
                        player_expression: Box::new(player_expression),
                        payload_expression: Box::new(payload_expression),
                        expression_range,
                    },
                    ParsedRobloxRemoteOperationKind::InvokeClient => Self::InvokeClient {
                        remote_type,
                        remote_expression: Box::new(remote_expression),
                        player_expression: Box::new(player_expression),
                        payload_expression: Box::new(payload_expression),
                        expression_range,
                    },
                    ParsedRobloxRemoteOperationKind::Connect
                    | ParsedRobloxRemoteOperationKind::Disconnect
                    | ParsedRobloxRemoteOperationKind::Wait
                    | ParsedRobloxRemoteOperationKind::FireServer
                    | ParsedRobloxRemoteOperationKind::FireAllClients
                    | ParsedRobloxRemoteOperationKind::InvokeServer
                    | ParsedRobloxRemoteOperationKind::SetCallback => return None,
                })
            }
            ParsedRobloxRemoteOperationKind::Wait if arguments.len() == 1 => {
                let remote_expression = arguments.pop()?;
                Some(Self::Wait {
                    remote_type: remote_type?,
                    remote_expression: Box::new(remote_expression),
                    expression_range,
                })
            }
            ParsedRobloxRemoteOperationKind::Connect
            | ParsedRobloxRemoteOperationKind::Disconnect
            | ParsedRobloxRemoteOperationKind::Wait
            | ParsedRobloxRemoteOperationKind::FireServer
            | ParsedRobloxRemoteOperationKind::FireClient
            | ParsedRobloxRemoteOperationKind::FireAllClients
            | ParsedRobloxRemoteOperationKind::InvokeServer
            | ParsedRobloxRemoteOperationKind::InvokeClient
            | ParsedRobloxRemoteOperationKind::SetCallback => None,
        }
    }

    pub(crate) const fn expression_range(&self) -> SourceRange {
        match self {
            Self::Connect {
                expression_range, ..
            }
            | Self::Disconnect {
                expression_range, ..
            }
            | Self::Wait {
                expression_range, ..
            }
            | Self::FireServer {
                expression_range, ..
            }
            | Self::FireClient {
                expression_range, ..
            }
            | Self::FireAllClients {
                expression_range, ..
            }
            | Self::InvokeServer {
                expression_range, ..
            }
            | Self::InvokeClient {
                expression_range, ..
            }
            | Self::SetCallback {
                expression_range, ..
            } => *expression_range,
        }
    }
}
