use crate::{checked_program::CheckedExpression, RemoteExecutionSide};

/// Retains a semantically validated operation at a direction-specific remote boundary.
pub enum CheckedRobloxRemoteOperation {
    /// Connects a callback to the selected `RemoteEvent` direction.
    Connect {
        /// `RemoteEvent` expression receiving the connection.
        remote_expression: Box<CheckedExpression>,
        /// Callback whose exact signature was checked against the event catalog.
        callback_expression: Box<CheckedExpression>,
        /// Runtime side selecting `OnServerEvent` or `OnClientEvent`.
        execution_side: RemoteExecutionSide,
    },
    /// Disconnects a checked `RBXScriptConnection`.
    Disconnect {
        /// Connection expression to disconnect.
        connection_expression: Box<CheckedExpression>,
    },
    /// Fires a `RemoteEvent` toward the server.
    FireServer {
        /// `RemoteEvent` expression being fired.
        remote_expression: Box<CheckedExpression>,
        /// Payload proven safe for the remote boundary.
        payload_expression: Box<CheckedExpression>,
    },
    /// Fires a `RemoteEvent` toward one client.
    FireClient {
        /// `RemoteEvent` expression being fired.
        remote_expression: Box<CheckedExpression>,
        /// Player expression selecting the receiving client.
        player_expression: Box<CheckedExpression>,
        /// Payload proven safe for the remote boundary.
        payload_expression: Box<CheckedExpression>,
    },
    /// Fires a `RemoteEvent` toward every client.
    FireAllClients {
        /// `RemoteEvent` expression being fired.
        remote_expression: Box<CheckedExpression>,
        /// Payload proven safe for the remote boundary.
        payload_expression: Box<CheckedExpression>,
    },
    /// Invokes a `RemoteFunction` on the server.
    InvokeServer {
        /// `RemoteFunction` expression being invoked.
        remote_expression: Box<CheckedExpression>,
        /// Payload proven safe for the remote boundary.
        payload_expression: Box<CheckedExpression>,
    },
    /// Invokes a `RemoteFunction` on one client.
    InvokeClient {
        /// `RemoteFunction` expression being invoked.
        remote_expression: Box<CheckedExpression>,
        /// Player expression selecting the receiving client.
        player_expression: Box<CheckedExpression>,
        /// Payload proven safe for the remote boundary.
        payload_expression: Box<CheckedExpression>,
    },
    /// Installs a callback on the module's `RemoteFunction` direction.
    SetCallback {
        /// `RemoteFunction` expression receiving the callback.
        remote_expression: Box<CheckedExpression>,
        /// Callback whose exact signature was checked against the function catalog.
        callback_expression: Box<CheckedExpression>,
        /// Runtime side selecting `OnServerInvoke` or `OnClientInvoke`.
        execution_side: RemoteExecutionSide,
    },
}
