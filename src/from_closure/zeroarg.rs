//! This implements metafactory construction from a zero-argument closure.
//!
//! Uses the same mechanism as manyarg closure, but is much easier to read,
//! because there are no macros here.
//!
//! ```
//! use metafactory::metafactory;
//! use metafactory::AsFactoryExt;
//!
//! fn main() {
//!     // build a metafactory from zero-argument closure.
//!     let meta_factory = metafactory(|| true);
//!
//!     // create a factory instance this closure.
//!     let factory = meta_factory
//!         .new(Vec::new()).ok().unwrap()
//!         .as_factory_of::<bool>().unwrap();
//!
//!     // value should match what factory produced.
//!     assert_eq!(true, factory.take());
//! }
//! ```

use std::any::Any;
use std::rc::Rc;
use std::cell::RefCell;

use typedef::TypeDef;

use super::super::{ MetaFactory, ToMetaFactory };
use super::super::factory::{ Factory, Getter };
use super::super::error::{ FactoryErrorKind };
use aggregate::Aggregate;

/// Creates `MetaFactory` from closure function.
#[stable]
impl<T:'static> ToMetaFactory for (||:'static -> T) {
    fn to_metafactory<'a>(self) -> Box<MetaFactory + 'a> {
        // We have only one closure, but the meta factory
        // will need to create many factories. There is no way around it
        // but put this single closure into reference-counted cell
        // so it can be uniquely dereferenced and called when each cloned Rc
        // is invoked in factory.
        box Rc::new(RefCell::new(self))
    }
}

/// Use closure itself as `MetaFactory`.
impl<T:'static> MetaFactory for Rc<RefCell<||:'static -> T>> {
    fn get_type(&self) -> TypeDef {
        TypeDef::of::<T>()
    }

    fn get_arg_types(&self) -> Vec<TypeDef> {
        Vec::new()
    }

    fn new(&self, _arg_getters: Vec<Box<Any>>) -> Result<Box<Any>, FactoryErrorKind> {
        Ok(
            box Factory::<T>::new(
                box self.clone()
            ) as Box<Any>
        )
    }

    fn new_aggregate(&self) -> Aggregate {
        Aggregate::new::<T>()
    }
}

/// And also use closure itself as created `Factory`.
impl<T: 'static> Getter<T> for Rc<RefCell<||:'static -> T>> {
    fn take(&self) -> T {
        (*(self.borrow_mut().deref_mut()))()
    }

    fn boxed_clone(&self) -> Box<Getter<T> + 'static> {
        box self.clone()
    }
}

#[cfg(test)]
mod test {
    use typedef::TypeDef;
    use super::super::super::{ ToMetaFactory, MetaFactory, AsFactoryExt }; // super

    #[test]
    fn should_return_correct_type() {
        assert_eq!(
            create(|| 24i).get_type(),
            TypeDef::of::<int>()
        );
        assert_eq!(
            create(|| 1f32).get_type(),
            TypeDef::of::<f32>()
        );
        assert_eq!(
            create(|| "aaa".to_string()).get_type(),
            TypeDef::of::<String>()
        );
        assert_eq!(
            create(|| box "aaa".to_string()).get_type(),
            TypeDef::of::<Box<String>>()
        );
    }

    #[test]
    fn should_require_no_arguments() {
        assert_eq!(
            create(|| 24i).get_arg_types().len(),
            0
        );
    }

    #[test]
    fn should_build_usable_factory() {
        assert_eq!(
            create(|| 24i).new(Vec::new()).ok().unwrap().as_factory_of::<int>().unwrap().take(),
            24i
        );
    }

    #[test]
    fn factory_clone_should_return_same_value() {
        let factory = create(|| 24i).new(Vec::new()).ok().unwrap().as_factory_of::<int>().unwrap();
        assert_eq!(
            factory.take(),
            factory.clone().take()
        );
    }

    fn create<'r, T: ToMetaFactory>(source: T) -> Box<MetaFactory + 'r> {
        source.to_metafactory()
    }
}
