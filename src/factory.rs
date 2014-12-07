use std::any::{ Any };
use std::boxed::{ BoxAny };

/// Gettable value trait.
#[experimental]
pub trait Getter<T> {
    fn get(&self) -> T;
}

/// A factory proxy.
///
/// `Factory` proxy is used to return a concrete value from
/// unknown source. `Factory` will always produce a new owned value -
/// any other pattern can be implemented on top of that.
#[stable]
pub struct Factory<'a, T> {
    getter: Box<Getter<T> + 'a>,
}

#[stable]
impl<'a, T: 'static> Factory<'a, T> {
    /// Create a new `Factory`.
    ///
    /// Create a new factory from any type that implements `Getter` trait.
    pub fn new(getter: Box<Getter<T> + 'a>) -> Factory<'a, T> {
        Factory::<T> {
            getter: getter,
        }
    }

    pub fn from_any(any: Box<Any>) -> Factory<'a, T> {
        let val = any
            .downcast::<Factory<T>>()
            .unwrap();
        *val
    }

    /// Get a new owned value.
    pub fn get(&self) -> T {
        self.getter.get()
    }
}
