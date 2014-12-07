use std::any::{ Any };
use std::boxed::{ BoxAny };

/// Gettable value trait.
#[experimental]
pub trait Getter<T> {
    fn get(&self) -> T;
    fn boxed_clone<'a>(&self) -> Box<Getter<T> + 'a>;
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

    /// Get a new owned value.
    pub fn get(&self) -> T {
        self.getter.get()
    }
}

impl<'a, T: 'static> Clone for Factory<'a, T> {
    fn clone(&self) -> Factory<'a, T> {
        Factory::<T> {
            getter: self.getter.boxed_clone(),
        }
    }
}

pub trait ToFactory {
    fn to_factory<'a, T>(self) -> Option<Factory<'a, T>>;
}

impl ToFactory for Box<Any> {
    fn to_factory<'a, T: 'static>(self) -> Option<Factory<'a, T>> {
        match self.downcast::<Factory<T>>().ok() {
            Some(val) => Some(*val),
            None => None
        }
    }
}

#[cfg(test)]
mod test {
    use super::{ Getter, Factory, ToFactory };
    use std::any::Any;

    #[test]
    fn should_get_correct_value() {
        let factory = create_with_val("HAI");

        assert_eq!(factory.get(), "HAI");
    }

    #[test]
    fn cloned_factory_should_get_the_same_value() {
        let factory = create_with_val("HAI");

        assert_eq!(factory.clone().get(), factory.get());
    }

    #[test]
    fn should_be_able_to_downcast_from_any() {
        let boxany = box create_with_val("HAI") as Box<Any>;
        let downcasted = boxany.to_factory::<String>().unwrap();

        assert_eq!(downcasted.get(), "HAI");
    }

    fn create_with_val(val: &str) -> Factory<String> {
        Factory::new(box ValContainer { val: val.to_string() })
    }

    struct ValContainer {
        val: String,
    }

    impl Getter<String> for ValContainer {
        fn get(&self) -> String {
            self.val.clone()
        }

        fn boxed_clone<'r>(&self) -> Box<Getter<String> + 'r> {
            box ValContainer {
                val: self.val.clone(),
            }
        }
    }
}
