//! Implements a factory that aggregates the results of other factories of
//! the same type.

use std::any::{ Any };
use std::boxed::BoxAny;
use typedef::TypeDef;
use factory::{ Factory, Getter };

/// Proxy for initializing aggregate factory without caring about the type used.
///
/// Its intended use is as a container of factories that produce a
/// value of the same type. Then it can itself be used to create a new factory
/// that aggregates values from all its childs and returns all of them
/// in a `Vec<T>` type for single `take`.
///
/// The most convenient way to initialize a new aggregate is to use
/// `new_aggregate` method on a metafactory. It will make a correctly typed
/// aggregate based on the metafactory type.
///
/// ```
/// # extern crate metafactory;
/// use metafactory::{ metafactory, AsFactoryExt };
///
/// fn main() {
///     let true_metafactory = metafactory(|| true);
///     let false_metafactory = metafactory(|| false);
///
///     // Use any of metafactories to create an aggregate for bool.
///     let mut aggregate = false_metafactory.new_aggregate();
///
///     // We can then add both factories to the aggregate.
///     // and make a factory from aggregate.
///     let true_and_false = aggregate
///         .new_factory(vec![
///             true_metafactory.new(Vec::new()).ok().unwrap(),
///             false_metafactory.new(Vec::new()).ok().unwrap(),
///         ])
///         .as_factory_of::<Vec<bool>>().unwrap();
///
///     assert_eq!(vec![true, false], true_and_false.take());
/// }
/// ```
///
/// The example bellow show how to initialize and use aggregate without a
/// metafactory.
///
/// ```
/// # extern crate metafactory;
/// use std::any::Any;
/// use metafactory::{ metafactory, argless_as_factory, AsFactoryExt };
/// use metafactory::aggregate::Aggregate;
///
/// fn main() {
///     // Let's say we know that we will have bunch of `bool` factories.
///     // In that case we create a new aggregate for them:
///     let mut aggregate = Aggregate::new::<bool>();
///
///     // Once we actually have our factories, we can inject them into aggregate
///     // without dealing with types. Of course, we should make sure that types
///     // actually match before doing that in the code that is using this
///     // implementation.
///     // Then we can call `new_factory` to convert all dynamic stuff to
///     // statically constructed call hierarchy:
///     let anyed_bool_array_factory = aggregate
///         .new_factory(vec![
///             argless_as_factory(|| true),
///             argless_as_factory(true),
///             argless_as_factory(|| 4i == 8),
///         ]);
///
///     // Of course, that returns it anyed (`Box<Any>`), but we can easily get un-anyed version
///     // by downcasting to `Factory<Vec<bool>>` or using a convenience extension
///     // method for that:
///     let bool_array_factory = anyed_bool_array_factory
///         .as_factory_of::<Vec<bool>>().unwrap();
///
///     // Calling it should produce expected boolean vector:
///     assert_eq!(bool_array_factory.take(), vec![true, true, false]);
///
///     // Of course, the aggregate itself should be usable as argument
///     // for other factories:
///     let metafactory_all_true = metafactory(|values: Vec<bool>| {
///         values.iter().fold(true, |a, &i| a & i)
///     });
///
///     // We can pass it when constructing a factory for this lambda metafactory:
///     let factory_all_true = metafactory_all_true.new(vec![
///         box bool_array_factory.clone() as Box<Any>
///     ])
///         .ok().unwrap() // check for errors here
///         .as_factory_of::<bool>().unwrap() // same story with downcasting
///     ;
///
///     assert_eq!(factory_all_true.take(), false); // not all values are true
/// }
/// ```
pub struct Aggregate<'a> {
    typedef: TypeDef,
    container_typedef: TypeDef,
    do_new: Box<Fn<(Vec<Box<Any>>,),Box<Any>> + 'a>,
}

impl<'a> Aggregate<'a> {
    /// Create new aggregate instance for specified type.
    pub fn new<T: 'static>() -> Aggregate<'a> {
        Aggregate {
            typedef: TypeDef::of::<T>(),
            container_typedef: TypeDef::of::<Vec<T>>(),
            do_new: box |&: items: Vec<Box<Any>>| {
                box Factory::<Vec<T>>::new(
                    box AggregateGetter::<T>::new(
                        items.into_iter()
                            .map(|i| *i.downcast::<Factory<T>>().ok().expect(
                                format!("failed to downcast factory child to Factory<{}>", TypeDef::name_of::<T>()).as_slice()
                            ))
                            .collect()
                    )
                )
            }
        }
    }

    /// Return aggregated type.
    pub fn get_arg_type(&self) -> TypeDef {
        self.typedef.clone()
    }

    /// Return container type.
    pub fn get_container_type(&self) -> TypeDef {
        self.container_typedef.clone()
    }

    /// Produces factory usable as argument for other factories.
    ///
    /// If inner factories make `int` values, this method will make factory
    /// that makes `Vec<int>` values.
    pub fn new_factory(&self, items: Vec<Box<Any>>) -> Box<Any> {
        (self.do_new).call((items,))
    }
}

struct AggregateGetter<T: 'static> {
    factories: Vec<Factory<T>>,
}

impl<T> Clone for AggregateGetter<T> {
    fn clone(&self) -> AggregateGetter<T> {
        AggregateGetter::<T> {
            factories: self.factories.clone()
        }
    }
}

impl<T> AggregateGetter<T> {
    pub fn new(factories: Vec<Factory<T>>) -> AggregateGetter<T> {
        AggregateGetter::<T> {
            factories: factories
        }
    }
}

impl<T> Getter<Vec<T>> for AggregateGetter<T> {
    fn take(&self) -> Vec<T> {

        // Reserve exact result size.
        let mut items = Vec::<T>::with_capacity(self.factories.len());

        // Construct results from factory results.
        items.extend(
            self.factories.iter()
                .map(|f| f.take())
        );

        items
    }

    fn boxed_clone(&self) -> Box<Getter<Vec<T>> + 'static> {
        box self.clone()
    }
}

#[cfg(test)]
mod test {
    use { argless_as_factory, metafactory, AsFactoryExt };
    use super::{ Aggregate };

    #[test]
    fn should_be_usable_as_vec_of_types() {
        let mut container = Aggregate::new::<int>();

        let parent_metafactory = metafactory(
            |items: Vec<int>|
                items.into_iter()
                    .map(|i| format!("{}", i))
                    .collect::<Vec<String>>()
                    .connect(", ")
        );

        let parent_getter = parent_metafactory
            .new(vec![
                container.new_factory(
                    vec![
                        argless_as_factory(5i),
                        argless_as_factory(13i)
                    ]
                )
            ]).ok().unwrap()
            .as_factory_of::<String>().unwrap()
        ;

        assert_eq!(parent_getter.take(), "5, 13");
    }
}
