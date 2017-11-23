/*!
<a href="https://github.com/Nercury/metafactory-rs">
    <img style="position: absolute; top: 0; left: 0; border: 0;" src="https://s3.amazonaws.com/github/ribbons/forkme_left_red_aa0000.png" alt="Fork me on GitHub">
</a>
<style>.sidebar { margin-top: 53px }</style>
*/

//!
//!
//!`MetaFactory` can be used to construct value creation chains from
//! closures or other sources that produce values.
//!
//! Let's look at really small example first:
//!
//! ```
//! use metafactory::{ metafactory, argless_as_factory, AsFactoryExt };
//!
//! fn main() {
//!     let meta_sum = metafactory(
//!         |a: int, b: int| a + b
//!     );
//!
//!     let sum_factory = meta_sum.new(vec![
//!         argless_as_factory(5i),
//!         argless_as_factory(6i),
//!     ]).ok().unwrap();
//!
//!     let getter = sum_factory.as_factory_of::<int>().unwrap();
//!
//!     assert_eq!(getter.take(), 11);
//! }
//! ```
//!
//! What is going on? Well, first we use `metafactory` generic function to
//! convert arbitrary closure to `MetaFactory` object `meta_sum`.
//!
//! It has a method `new`, which is used above to return a real
//! concrete factory `sum_factory`. As argument, it takes other factories.
//! The method `argless_as_factory()` returns factories for clonable values
//! `5i` and `6i`.
//!
//! So, metafactories can be created from different sources: clonable
//! objects or closures. In this case `5i` is a clonable object.
//!
//! Returned `sum_factory` has a `Box<Any>` type, and can be downcasted to
//! a `Factory` of appropriate type with `as_factory_of` method.
//!
//! Then you can call `take()` on this factory `getter` to invoke your closure
//! and return a new value.
//!
//! Note a lot of `unwrap` calls. All the potential argument or type
//! mismatches produce correct errors that can be dealt with.
//!
//! Now, the example above might seem a bit involved, but consider what it
//! allows us to do: provided argument counts and types match, we can
//! connect any chain of factories together, at runtime.
//!
//! Let's expand example to add a new factory that uses `sum_factory` as an
//! argument, and creates our own struct:
//!
//! ```
//! use metafactory::{ metafactory, argless_as_factory, AsFactoryExt };
//!
//! /// Our own struct.
//! struct Foo {
//!     value: int,
//! }
//!
//! fn main() {
//!     // initialization
//!
//!     let meta_sum = metafactory(
//!         |a: int, b: int| a + b
//!     );
//!
//!     let meta_foo = metafactory(
//!         |sum: int| Foo { value: sum }
//!     );
//!
//!     // plugging in
//!
//!     let foo_factory = meta_foo.new(vec![
//!         meta_sum.new(vec![
//!             argless_as_factory(5i),
//!             argless_as_factory(6i),
//!         ]).ok().unwrap()
//!     ]).ok().unwrap();
//!
//!     // using
//!
//!     let getter = foo_factory.as_factory_of::<Foo>().unwrap();
//!
//!     assert_eq!(getter.take().value, 11);
//! }
//! ```
//!
//! So, the intention of this library is to have a mechanism to separate 3
//! things: creating the values, plugging-in the factories
//! values, and using them.
//!
//! Created metafactories can also be used to inspect closure argument types.
//! It is also possible to clone the created factory: in such case, all
//! the call tree is also cloned internally. This makes it possible to
//! pass a copy of a factory to a task.
//!
//! Factories intentionally do not produce any kind of singletons or
//! references, only new values. You can also look at them as configurable
//! stream of values.
//!
//! If this library looks a bit lower-level, it is because it is intended as
//! such: more convenient wrappers like dependency injection or plugin
//! architecture can be implemented on top of this.
//!
//! Finally, a more complete example of available functionality:
//!
//! ```
//! use metafactory::{ metafactory, AsFactoryExt };
//!
//! fn main() {
//!     // build argument-factory from cloneable source.
//!     let meta_arg1 = metafactory(5i);
//!
//!     // build argument-factory from lambda.
//!     let meta_arg2 = metafactory(|| 14i32);
//!
//!     // build a factory that uses other factories create added numbers.
//!     let meta_adder = metafactory(|a1: int, a2: i32| a1 + a2 as int);
//!
//!     // it knows the cloneable source returns int
//!     assert!(meta_arg1.get_type().is::<int>());
//!     // it knows the lambda source returns i32
//!     assert!(meta_arg2.get_type().is::<i32>());
//!     // it knows the lambda with 2 args returns int
//!     assert!(meta_adder.get_type().is::<int>());
//!
//!     // create a factory for adder, pass other 2 factories as arguments
//!     let boxany = meta_adder.new(vec![
//!         meta_arg1.new(Vec::new()).ok().unwrap(),
//!         meta_arg2.new(Vec::new()).ok().unwrap(),
//!     ]).ok().unwrap();
//!
//!     // conveniently downcast factory to callable instance
//!     let factory = boxany.as_factory_of::<int>().unwrap();
//!
//!     // value should be the sum.
//!     assert_eq!(19, factory.take());
//!
//!     // factory can be cloned
//!     let factory2 = factory.clone();
//!
//!     // both clones invoke the same construction path and return the same value
//!     assert_eq!(factory.take(), factory2.take());
//! }
//! ```

#![feature(macro_rules)]
#![feature(unboxed_closures)]

extern crate typedef;

use std::any::{ Any };
use std::boxed::{ BoxAny };

use typedef::{ TypeDef };
use error::{ FactoryErrorKind };
use aggregate::Aggregate;

pub mod aggregate;
pub mod error;

mod factory;
mod from_clone;
mod from_closure;

/// Gettable value trait.
#[experimental]
pub trait Getter<T> {
    /// Produce a new value.
    fn take(&self) -> T;

    /// Create a clone for this getter.
    ///
    /// This is kind of experimental solution - can not return plain traits
    /// as function result.
    fn boxed_clone(&self) -> Box<Getter<T> + 'static>;
}

/// A factory proxy.
///
/// `Factory` proxy is used to return a concrete value from
/// unknown source. `Factory` will always produce a new owned value -
/// any other pattern can be implemented on top of that.
#[stable]
pub struct Factory<T:'static> {
    getter: Box<Getter<T> + 'static>,
}

#[stable]
impl<'a, T: 'static> Factory<T> {
    /// Create a new `Factory`.
    ///
    /// Create a new factory from any type that implements `Getter` trait.
    pub fn new(getter: Box<Getter<T> + 'static>) -> Factory<T> {
        Factory::<T> {
            getter: getter,
        }
    }

    /// Get a new owned value.
    pub fn take(&self) -> T {
        self.getter.take()
    }
}

impl<'a, T: 'static> Clone for Factory<T> {
    fn clone(&self) -> Factory<T> {
        Factory::<T> {
            getter: self.getter.boxed_clone(),
        }
    }
}

/// Downcast value to `Factory`.
pub trait AsFactoryExt {
    /// Downcast to factory and consume `Box<Any>`.
    fn as_factory_of<T>(self) -> Option<Factory<T>>;
    /// Downcast to factory by creating factory clone.
    fn as_factory_clone_of<T>(&self) -> Option<Factory<T>>;
}

impl AsFactoryExt for Box<Any> {
    fn as_factory_of<'a, T: 'static>(self) -> Option<Factory<T>> {
        match self.downcast::<Factory<T>>().ok() {
            Some(val) => Some(*val),
            None => None
        }
    }

    fn as_factory_clone_of<'a, T: 'static>(&self) -> Option<Factory<T>> {
        match self.downcast_ref::<Factory<T>>() {
            Some(val) => Some(val.clone()),
            None => None
        }
    }
}

/// Implements reflection and initiation of any abstract object constructor.
///
/// ## Information about constructor
///
/// The first goal of this trait is being able to inspect the construction
/// mechanism at the runtime.
///
/// `MetaFactory` contains information about the constructor source: the
/// return type and argument types. It also contains a method `new`
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
/// can be invoked with a simple `take()` call.
///
/// ## Simple value as constructor
///
/// This library allows to use many sources as "constructors", the simplest
/// of which is a cloneable value. In this example we will use such value to
/// create a new constructor metadata, check if argument and return types are
/// correct, and then create an actual getter for the value:
///
/// ```
/// use metafactory::{ metafactory, AsFactoryExt };
///
/// let metafactory = metafactory(5i);
/// assert!(metafactory.get_type().is::<int>());
/// assert!(metafactory.get_arg_types().len() == 0); // clonable int has no arguments
///
/// let factory = metafactory
///     .new(
///         Vec::new() // No arguments in this case.
///     )
///     .ok().unwrap()
///     .as_factory_of::<int>()
///     .unwrap();
///
/// assert_eq!(factory.take(), 5i);
/// ```
#[unstable]
pub trait MetaFactory {
    #[unstable]
    fn get_type(&self) -> TypeDef;
    #[unstable]
    fn get_arg_types(&self) -> Vec<TypeDef>;
    #[unstable]
    fn new(&self, arg_getters: Vec<Box<Any>>) -> Result<Box<Any>, FactoryErrorKind>;
    #[unstable]
    fn new_aggregate(&self) -> Aggregate<'static>;
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
pub fn metafactory<'r, T: ToMetaFactory>(any: T) -> Box<MetaFactory + 'r> {
    any.to_metafactory()
}

/// Create a new `MetaFactory` and return `Factory` in `Box<Any>` for source with no arguments.
///
/// Compatible value type must have `ToMetaFactory` implementation.
/// Supported sources are in submodules, look at "clone" for simpliest example.
pub fn argless_as_factory<T: ToMetaFactory>(any: T) -> Box<Any> {
    any.to_metafactory().new(Vec::new()).ok().unwrap()
}
