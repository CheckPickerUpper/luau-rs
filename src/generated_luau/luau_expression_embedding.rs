use crate::generated_luau::{LuauExpressionPrecedence, LuauOperationOperandSide};

/// Describes where an expression is written so parentheses preserve its meaning.
#[derive(Clone, Copy)]
pub enum LuauExpressionEmbedding {
    /// Writes an expression where no enclosing operator can change its meaning.
    Unrestricted,
    /// Writes an expression as a binary operation operand.
    OperationOperand {
        /// Gives the enclosing operation's binding strength.
        parent_precedence: LuauExpressionPrecedence,
        /// Gives the operand position that determines source association.
        operand_side: LuauOperationOperandSide,
    },
    /// Writes an expression inside a function-call argument list.
    FunctionArgument,
}
