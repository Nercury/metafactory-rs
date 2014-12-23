//! Implements a factory that aggregates the results of other factories of
//! the same type.

use std::any::{ Any, AnyMutRefExt };
use std::boxed::BoxAny;
use typedef::TypeDef;
use factory::{ Factory, Getter };

/// Proxy for initializing aggregate factory without caring about the type used.
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
///     aggregate.push_items(vec![
///         argless_as_factory(|| true),
///         argless_as_factory(true),
///         argless_as_factory(|| 4i == 8),
///     ]);
///
///     // Once we are ready to use the aggregate as factory, we can call `new_factory` to
///     // convert all dynamic stuff to statically constructed call hierarchy:
///     let anyed_bool_array_factory = aggregate.new_factory();
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
    any_getter: Box<Any>,
    do_push_items: |&mut Box<Any>, Vec<Box<Any>>|:'a -> (),
    do_new_factory: |&mut Box<Any>|:'a -> Box<Any>, // Don't worry, it's like Javascript ;)
}

impl<'a> Aggregate<'a> {
    /// Create new aggregate instance for specified type.
    pub fn new<T: 'static>() -> Aggregate<'a> {
        Aggregate {
            typedef: TypeDef::of::<T>(),
            any_getter: box AggregateGetter::<T>::new(),
            do_push_items: |any_getter, items| {
                let getter: &mut AggregateGetter<T> = any_getter
                    .downcast_mut::<AggregateGetter<T>>().unwrap();

                let len = items.len();
                let items_iter = items.into_iter()
                    .map(|i| *i.downcast::<Factory<T>>().ok().unwrap());

                getter.factories.reserve_exact(len);
                getter.factories.extend(items_iter);
            },
            do_new_factory: |any_getter| {
                let getter: &mut AggregateGetter<T> = any_getter
                    .downcast_mut::<AggregateGetter<T>>().unwrap();

                box Factory::<Vec<T>>::new(getter.boxed_clone())
            }
        }
    }

    /// Return aggregated type.
    pub fn get_arg_type(&self) -> TypeDef {
        self.typedef.clone()
    }

    /// Push factory items into aggregate.
    ///
    /// Note that all items should already match contained aggregate type:
    /// if aggregate was created for `int`, all pushed factories should
    /// produce int. Otherwise this method will panic your app.
    pub fn push_items(&mut self, items: Vec<Box<Any>>) {
        (self.do_push_items)(&mut self.any_getter, items);
    }

    /// Produces factory usable as argument for other factories.
    ///
    /// If inner factories make `int` values, this method will make factory
    /// that makes `Vec<int>` values.
    pub fn new_factory(&mut self) -> Box<Any> {
        (self.do_new_factory)(&mut self.any_getter)
    }

    /// Returns type which is used as aggregate result.
    pub fn container_of<T: 'static>() -> TypeDef {
        TypeDef::of::<Vec<T>>()
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
    pub fn new() -> AggregateGetter<T> {
        AggregateGetter::<T> {
            factories: Vec::with_capacity(0)
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
        container.push_items(vec![
            argless_as_factory(5i),
            argless_as_factory(13i)
        ]);

        let parent_metafactory = metafactory(
            |items: Vec<int>|
                items.into_iter()
                    .map(|i| format!("{}", i))
                    .collect::<Vec<String>>()
                    .connect(", ")
        );

        let parent_getter = parent_metafactory
            .new(vec![
                container.new_factory()
            ]).ok().unwrap()
            .as_factory_of::<String>().unwrap()
        ;

        assert_eq!(parent_getter.take(), "5, 13");
    }
}
