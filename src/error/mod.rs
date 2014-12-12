//! Closure argument error types.

use typedef::TypeDef;

/// Specified argument count does not match metafactory argument count.
#[deriving(Copy)]
pub struct ArgCountMismatch {
    pub expected: uint,
    pub specified: uint,
}

/// Argument type did not match expected type.
#[deriving(Copy)]
pub struct ArgTypeMismatch {
    pub expected_type: TypeDef,
    pub argument_index: uint,
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

impl ArgTypeMismatch {
    /// Convenience method for creating new `ArgTypeMismatch`.
    pub fn new(expected_type: TypeDef, argument_index: uint) -> ArgTypeMismatch {
        ArgTypeMismatch {
            expected_type: expected_type,
            argument_index: argument_index,
        }
    }
}

/// Getter creation error types.
#[deriving(Copy)]
pub enum FactoryErrorKind {
    /// Incorrect number of arguments.
    ArgCountMismatch(ArgCountMismatch),
    /// Incorrect argument type.
    ArgTypeMismatch(ArgTypeMismatch),
}
