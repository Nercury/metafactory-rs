use typedef::{ TypeDef };
use std::any::{ Any };

use super::{ MetaFactory, ToMetaFactory };
use super::factory::{ Factory, Getter };

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

    fn boxed_clone(&self) -> Box<Getter<T> + 'static> {
        box CloneableValue::<T> { value: self.value.clone() }
    }
}

#[cfg(test)]
mod test {
    use typedef::TypeDef;
    use super::super::{ ToMetaFactory, MetaFactory };

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
        use super::super::factory::{ ToFactory };
        assert_eq!(
            create(24i).new_factory(Vec::new()).to_factory::<int>().unwrap().get(),
            24i
        );
    }

    #[test]
    fn factory_clone_should_return_same_value() {
        use super::super::factory::{ ToFactory };
        let factory = create(24i).new_factory(Vec::new()).to_factory::<int>().unwrap();
        assert_eq!(
            factory.get(),
            factory.clone().get()
        );
    }

    fn create<'r, T: ToMetaFactory>(source: T) -> Box<MetaFactory + 'r> {
        source.to_metafactory()
    }
}
