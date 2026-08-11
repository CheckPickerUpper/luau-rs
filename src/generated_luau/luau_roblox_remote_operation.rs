use crate::{
    generated_luau::LuauExpression, remote_payload_shape::RemotePayloadShape, RemoteExecutionSide,
};

/// Owns the target-shaped Luau form of one checked remote operation.
#[derive(Debug, PartialEq, Eq)]
pub enum LuauRobloxRemoteOperation {
    /// Calls `OnServerEvent:Connect` or `OnClientEvent:Connect`.
    Connect {
        /// `RemoteEvent` expression receiving the connection.
        remote_expression: Box<LuauExpression>,
        /// Typed callback expression.
        callback_expression: Box<LuauExpression>,
        /// Runtime side selecting the event member.
        execution_side: RemoteExecutionSide,
        /// Runtime payload validation required before invoking the callback.
        payload_shape: RemotePayloadShape,
    },
    /// Calls `RBXScriptConnection:Disconnect`.
    Disconnect {
        /// Connection expression to disconnect.
        connection_expression: Box<LuauExpression>,
    },
    /// Calls `OnClientEvent:Wait` at an explicit yielding boundary.
    Wait {
        /// `RemoteEvent` expression whose client event yields.
        remote_expression: Box<LuauExpression>,
    },
    /// Calls `RemoteEvent:FireServer`.
    FireServer {
        /// `RemoteEvent` expression being fired.
        remote_expression: Box<LuauExpression>,
        /// Validated payload expression.
        payload_expression: Box<LuauExpression>,
    },
    /// Calls `RemoteEvent:FireClient`.
    FireClient {
        /// `RemoteEvent` expression being fired.
        remote_expression: Box<LuauExpression>,
        /// Target Player expression.
        player_expression: Box<LuauExpression>,
        /// Validated payload expression.
        payload_expression: Box<LuauExpression>,
    },
    /// Calls `RemoteEvent:FireAllClients`.
    FireAllClients {
        /// `RemoteEvent` expression being fired.
        remote_expression: Box<LuauExpression>,
        /// Validated payload expression.
        payload_expression: Box<LuauExpression>,
    },
    /// Calls `RemoteFunction:InvokeServer`.
    InvokeServer {
        /// `RemoteFunction` expression being invoked.
        remote_expression: Box<LuauExpression>,
        /// Validated payload expression.
        payload_expression: Box<LuauExpression>,
    },
    /// Calls `RemoteFunction:InvokeClient`.
    InvokeClient {
        /// `RemoteFunction` expression being invoked.
        remote_expression: Box<LuauExpression>,
        /// Target Player expression.
        player_expression: Box<LuauExpression>,
        /// Validated payload expression.
        payload_expression: Box<LuauExpression>,
    },
    /// Assigns `OnServerInvoke` or `OnClientInvoke`.
    SetCallback {
        /// `RemoteFunction` expression receiving the callback.
        remote_expression: Box<LuauExpression>,
        /// Typed callback expression.
        callback_expression: Box<LuauExpression>,
        /// Runtime side selecting the invoke member.
        execution_side: RemoteExecutionSide,
        /// Runtime payload validation required before invoking the callback.
        payload_shape: RemotePayloadShape,
    },
}
