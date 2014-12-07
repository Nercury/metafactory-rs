use std::any::Any;
use std::rc::Rc;
use std::cell::RefCell;

use typedef::TypeDef;

use super::super::{ MetaFactory, ToMetaFactory };
use super::super::factory::{ Factory, Getter };

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

impl<T:'static> MetaFactory for Rc<RefCell<||:'static -> T>> {
    fn get_type(&self) -> TypeDef {
        TypeDef::of::<T>()
    }

    fn get_arg_types(&self) -> Vec<TypeDef> {
        Vec::new()
    }

    fn new_factory(&self, _arg_getters: Vec<Box<Any>>) -> Box<Any> {
        box Factory::<T>::new(
            box self.clone()
        ) as Box<Any>
    }
}

impl<T: 'static> Getter<T> for Rc<RefCell<||:'static -> T>> {
    fn get(&self) -> T {
        (*(self.borrow_mut().deref_mut()))()
    }

    fn boxed_clone<'r>(&self) -> Box<Getter<T> + 'r> {
        box self.clone()
    }
}

#[cfg(test)]
mod test {
    use typedef::TypeDef;
    use super::super::super::{ ToMetaFactory, MetaFactory }; // super

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
        use super::super::super::factory::{ ToFactory };
        assert_eq!(
            create(|| 24i).new_factory(Vec::new()).to_factory::<int>().unwrap().get(),
            24i
        );
    }

    #[test]
    fn factory_clone_should_return_same_value() {
        use super::super::super::factory::{ ToFactory };
        let factory = create(|| 24i).new_factory(Vec::new()).to_factory::<int>().unwrap();
        assert_eq!(
            factory.get(),
            factory.clone().get()
        );
    }

    fn create<'r, T: ToMetaFactory>(source: T) -> Box<MetaFactory + 'r> {
        source.to_metafactory()
    }
}
