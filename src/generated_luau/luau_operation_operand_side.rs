/// Identifies which operand must retain a binary operation's source association.
#[derive(Clone, Copy)]
pub(crate) enum LuauOperationOperandSide {
    /// Identifies the operand evaluated before the operator.
    Left,
    /// Identifies the operand evaluated after the operator.
    Right,
}
