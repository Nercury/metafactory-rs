//! `MetaFactory` is used to build efficient object creation trees at
//! runtime. This is potentialy useful if the configuration of this
//! tree occurs in different libraries.
//!
//! In other words, it allows us to separate __what__ is created from
//! __how__ it is created.
//!
//! ```
//! use metafactory::new_metafactory;
//! use metafactory::factory::Factory;
//!
//! fn main() {
//!     // build meta-info for constructor from cloneable source.
//!     let mf = new_metafactory(5i);
//!
//!     // inspect the metafactory
//!     assert!(mf.get_type().is::<int>());
//!
//!     // create a factory that can be used as argument for other factory
//!     let boxany = mf.new_factory(Vec::new());
//!
//!     // conveniently donwcast factory to callable instance
//!     let factory = Factory::<int>::from_any(boxany);
//!
//!     // factory can be cloned
//!     let factory2 = factory.clone();
//!
//!     // both clones invoke the same construction path and return the same value
//!     assert_eq!(factory.get(), factory2.get());
//! }
//! ```

extern crate typedef;

use std::any::{ Any };
use typedef::{ TypeDef };

pub mod factory;
pub mod from_clone;

/// Implements reflection and initiation of any abstract object constructor.
///
/// ## Information about constructor
///
/// The first goal of this trait is being able to inspect the construction
/// mechanism at the runtime.
///
/// `MetaFactory` contains information about the constructor source: the
/// return type and argument types. It also contains a method `new_factory`
/// that can create a function to invoke this source and get the values.
///
/// ## Factory initiation
///
/// The second goal is to avoid donwcasting construction arguments on every
/// `Factory` invocation.
///
/// There are separate traits for `MetaFactory` and "real" `Factory`.
/// `MetaFactory` is used to build a real `Factory` by passing all the
/// required constructor arguments as factories under `Vec<Box<Any>>`.
///
/// Internaly, all parent `Factory`s will be downcasted to correct types
/// and stored inside returned `Factory`'s scope, so that all of them
/// can be invoked with a simple `get()` call.
///
/// ## Simple value as constructor
///
/// This library allows to use many sources as "constructors", the simplest
/// of which is a cloneable value. In this example we will use such value to
/// create a new constructor metadata, check if argument and return types are
/// correct, and then create an actual getter for the value:
///
/// ```
/// use metafactory::{ new_metafactory };
/// use metafactory::factory::{ Factory };
/// use std::boxed::BoxAny; // for downcast
///
/// let metafactory = new_metafactory(5i);
/// assert!(metafactory.get_type().is::<int>());
/// assert!(metafactory.get_arg_types().len() == 0); // clonable int has no arguments
///
/// let factory = metafactory
///     .new_factory(
///         Vec::new() // No arguments in this case.
///     )
///     .downcast::<Factory<int>>()
///     .unwrap();
///
/// assert_eq!(factory.get(), 5i);
/// ```
#[unstable]
pub trait MetaFactory {
    #[unstable]
    fn get_type(&self) -> TypeDef;
    #[unstable]
    fn get_arg_types(&self) -> Vec<TypeDef>;
    #[unstable]
    fn new_factory(&self, arg_getters: Vec<Box<Any>>) -> Box<Any>;
}

/// Trait for values convertable to `MetaFactory`.
///
/// This trait is implemented for values that can be used as
/// sources for object creation.
#[unstable]
pub trait ToMetaFactory {
    /// Creates a `MetaFactory` that has information about object
    /// constructor: produced object type, argument types, and
    /// a method to get this getter.
    #[unstable]
    fn to_metafactory<'a>(self) -> Box<MetaFactory + 'a>;
}

/// Create a new `MetaFactory` for any compatible value source.
///
/// Compatible value type must have `ToMetaFactory` implementation.
/// Supported sources are in submodules, look at "clone" for simpliest example.
pub fn new_metafactory<'r, T: ToMetaFactory>(any: T) -> Box<MetaFactory + 'r> {
    any.to_metafactory()
}
