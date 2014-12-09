//! Implements a wrapper struct container for internal getter.

use std::any::{ Any };
use std::boxed::{ BoxAny };

/// Gettable value trait.
#[experimental]
pub trait Getter<T> {
    /// Produce a new value.
    fn take(&self) -> T;

    /// Create a clone for this getter.
    ///
    /// This is kind of experimental solution - it allocates a new box
    /// to avoid breaking `Sized` requirement for `Factory::Clone`.
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
    fn as_factory_of<T>(self) -> Option<Factory<T>>;
}

impl AsFactoryExt for Box<Any> {
    fn as_factory_of<'a, T: 'static>(self) -> Option<Factory<T>> {
        match self.downcast::<Factory<T>>().ok() {
            Some(val) => Some(*val),
            None => None
        }
    }
}

#[cfg(test)]
mod test {
    use super::{ Getter, Factory };
    use super::AsFactoryExt;
    use std::any::Any;

    #[test]
    fn should_get_correct_value() {
        let factory = create_with_val("HAI");

        assert_eq!(factory.take(), "HAI");
    }

    #[test]
    fn cloned_factory_should_get_the_same_value() {
        let factory = create_with_val("HAI");

        assert_eq!(factory.clone().take(), factory.take());
    }

    #[test]
    fn should_be_able_to_downcast_from_any() {
        let boxany = box create_with_val("HAI") as Box<Any>;
        let downcasted = boxany.as_factory_of::<String>().unwrap();

        assert_eq!(downcasted.take(), "HAI");
    }

    fn create_with_val(val: &str) -> Factory<String> {
        Factory::new(box ValContainer { val: val.to_string() })
    }

    struct ValContainer {
        val: String,
    }

    impl Getter<String> for ValContainer {
        fn take(&self) -> String {
            self.val.clone()
        }

        fn boxed_clone<'r>(&self) -> Box<Getter<String> + 'r> {
            box ValContainer {
                val: self.val.clone(),
            }
        }
    }
}
