/// Specified argument count does not match metafactory argument count.
pub struct ArgCountMismatch {
    pub expected: uint,
    pub specified: uint,
}

impl ArgCountMismatch {
    /// Convenience method for creating new `ArgCountMismatch`.
    pub fn new(expected: uint, specified: uint) -> ArgCountMismatch {
        ArgCountMismatch {
            expected: expected,
            specified: specified,
        }
    }
}

/// Getter creation error types.
pub enum FactoryErrorKind {
    /// Incorrect number of mapped argument definitions.
    ArgCountMismatch(ArgCountMismatch),
}
