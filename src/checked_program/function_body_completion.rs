/// Names whether normal execution can continue after a checked body.
/// It lets value-returning functions prove that every reachable branch returns.
#[derive(Clone, Copy)]
pub enum FunctionBodyCompletion {
    /// At least one reachable path reaches the end of the body.
    ReachesEnd,
    /// Every reachable path returns from the enclosing function.
    AlwaysReturns,
}
