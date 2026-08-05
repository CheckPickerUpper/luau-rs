/// Names the boolean literals emitted into Luau expressions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LuauBooleanLiteral {
    /// Emits Luau's `true` literal.
    True,
    /// Emits Luau's `false` literal.
    False,
}
