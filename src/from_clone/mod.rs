//! This implements metafactory construction from cloneable value.
//!
//! ```
//! use metafactory::metafactory;
//! use metafactory::AsFactoryExt;
//!
//! fn main() {
//!     // build a metafactory from cloneable value.
//!     let meta_factory = metafactory("hello");
//!
//!     // create a factory instance this closure.
//!     let factory = meta_factory
//!         .new(Vec::new()).ok().unwrap()
//!         .as_factory_of::<&str>().unwrap();
//!
//!     // value should match what factory produced.
//!     assert_eq!("hello", factory.take());
//! }
//! ```

use typedef::{ TypeDef };
use std::any::{ Any };

use super::{ MetaFactory, ToMetaFactory };
use super::factory::{ Factory, Getter };
use super::error::{ FactoryErrorKind };
use aggregate::Aggregate;

#[experimental]
pub struct CloneableMetaFactory<T> {
    pub value: T,
}

#[experimental]
pub struct CloneableValue<T> {
    pub value: T,
}

/// Creates `MetaFactory` for cloneable value.
#[stable]
impl<T: 'static + Clone> ToMetaFactory for T {
    fn to_metafactory<'a>(self) -> Box<MetaFactory + 'a> {
        box CloneableMetaFactory { value : self }
    }
}

impl<T: 'static + Clone> MetaFactory for CloneableMetaFactory<T> {
    fn get_type(&self) -> TypeDef {
        TypeDef::of::<T>()
    }

    fn get_arg_types(&self) -> Vec<TypeDef> {
        Vec::new()
    }

    fn new(&self, _arg_getters: Vec<Box<Any>>) -> Result<Box<Any>, FactoryErrorKind> {
        Ok(
            box Factory::new(
                box CloneableValue::<T> { value: self.value.clone() }
            ) as Box<Any>
        )
    }

    fn new_aggregate(&self) -> Aggregate {
        Aggregate::new::<T>()
    }
}

impl<T: 'static + Clone> Getter<T> for CloneableValue<T> {
    fn take(&self) -> T {
        self.value.clone()
    }

    fn boxed_clone(&self) -> Box<Getter<T> + 'static> {
        box CloneableValue::<T> { value: self.value.clone() }
    }
}

#[cfg(test)]
mod test {
    use typedef::TypeDef;
    use super::super::{ ToMetaFactory, MetaFactory, AsFactoryExt };

    #[test]
    fn should_return_correct_type() {
        assert_eq!(
            create(24i).get_type(),
            TypeDef::of::<int>()
        );
        assert_eq!(
            create(1f32).get_type(),
            TypeDef::of::<f32>()
        );
        assert_eq!(
            create("aaa".to_string()).get_type(),
            TypeDef::of::<String>()
        );
        assert_eq!(
            create(box "aaa".to_string()).get_type(),
            TypeDef::of::<Box<String>>()
        );
    }

    #[test]
    fn should_require_no_arguments() {
        assert_eq!(
            create(24i).get_arg_types().len(),
            0
        );
    }

    #[test]
    fn should_build_usable_factory() {
        assert_eq!(
            create(24i).new(Vec::new()).ok().unwrap().as_factory_of::<int>().unwrap().take(),
            24i
        );
    }

    #[test]
    fn factory_clone_should_return_same_value() {
        let factory = create(24i).new(Vec::new()).ok().unwrap().as_factory_of::<int>().unwrap();
        assert_eq!(
            factory.take(),
            factory.clone().take()
        );
    }

    fn create<'r, T: ToMetaFactory>(source: T) -> Box<MetaFactory + 'r> {
        source.to_metafactory()
    }
}
