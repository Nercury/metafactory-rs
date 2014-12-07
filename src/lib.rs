use std::any::{ Any };
use typedef::{ TypeDef };

#[unstable]
pub trait MetaFactory {
    #[unstable]
    fn get_type(&self) -> TypeDef;
    #[unstable]
    fn get_arg_types(&self) -> Vec<TypeDef>;
    #[unstable]
    fn new_getter(&self, arg_getters: Vec<Box<Any>>) -> Box<Any>;
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

/// Gettable value trait.
#[experimental]
pub trait Gettable<T> {
    fn get(&self) -> T;
}

/// A gettable value proxy.
///
/// `ValueGetter` proxy is used to return a concrete value from
/// unknown source. `ValueGetter` will always produce a new owned value -
/// any other pattern can be implemented on top of that.
#[stable]
pub struct ValueGetter<'a, T> {
    getter: Box<Gettable<T> + 'a>,
}

#[stable]
impl<'a, T> ValueGetter<'a, T> {
    /// Create a new `ValueGetter`.
    ///
    /// Create a new getter from any type that implements `Gettable` trait.
    pub fn new(getter: Box<Gettable<T> + 'a>) -> ValueGetter<'a, T> {
        ValueGetter::<T> {
            getter: getter,
        }
    }

    /// Get a new owned value.
    pub fn get(&self) -> T {
        self.getter.get()
    }
}
