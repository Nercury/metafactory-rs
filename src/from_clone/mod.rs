use typedef::{ TypeDef };
use std::any::{ Any };

use super::{ MetaFactory, ToMetaFactory };
use super::factory::{ Factory, Getter };

#[deriving(Clone)]
#[experimental]
pub struct CloneableMetaFactory<T> {
    pub value: T,
}

#[deriving(Clone)]
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

    fn new_factory(&self, _arg_getters: Vec<Box<Any>>) -> Box<Any> {
        box Factory::new(
            box CloneableValue::<T> { value: self.value.clone() }
        ) as Box<Any>
    }
}

impl<T: 'static + Clone> Getter<T> for CloneableValue<T> {
    fn get(&self) -> T {
        self.value.clone()
    }

    fn boxed_clone<'a>(&self) -> Box<Getter<T> + 'a> {
        box CloneableValue::<T> { value: self.value.clone() }
    }
}
