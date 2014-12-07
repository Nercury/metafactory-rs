use std::any::Any;
use std::rc::Rc;
use std::cell::RefCell;

use typedef::TypeDef;

use super::super::{ MetaFactory, ToMetaFactory };
use super::super::factory::{ Factory, ToFactory, Getter };

#[experimental]
pub struct GetterScope1<A1:'static, T:'static> {
    a1: Factory<A1>,
    closure: Rc<RefCell<|A1|:'static -> T>>,
}

#[experimental]
pub struct GetterScope2<A1:'static, A2:'static, T:'static> {
    a1: Factory<A1>,
    a2: Factory<A2>,
    closure: Rc<RefCell<|A1, A2|:'static -> T>>,
}

/// Creates `MetaFactory` from closure function.
#[stable]
impl<A1:'static, T:'static> ToMetaFactory for (|A1|:'static -> T) {
    fn to_metafactory<'a>(self) -> Box<MetaFactory + 'a> {
        box Rc::new(RefCell::new(self))
    }
}

/// Creates `MetaFactory` from closure function.
#[stable]
impl<A1:'static, A2:'static, T:'static> ToMetaFactory for (|A1, A2|:'static -> T) {
    fn to_metafactory<'a>(self) -> Box<MetaFactory + 'a> {
        box Rc::new(RefCell::new(self))
    }
}

impl<A1:'static, T:'static> MetaFactory for Rc<RefCell<|A1|:'static -> T>> {
    fn get_type(&self) -> TypeDef {
        TypeDef::of::<T>()
    }

    fn get_arg_types(&self) -> Vec<TypeDef> {
        vec![TypeDef::of::<A1>()]
    }

    fn new_factory(&self, arg_getters: Vec<Box<Any>>) -> Box<Any> {
        let mut getters = arg_getters;
        box Factory::<T>::new(
            box GetterScope1::<A1, T> {
                a1: getters.pop().expect("arg 1 missing").to_factory::<A1>().expect("bad arg 1"),
                closure: self.clone(),
            }
        ) as Box<Any>
    }
}

impl<A1:'static, A2:'static, T:'static> MetaFactory for Rc<RefCell<|A1, A2|:'static -> T>> {
    fn get_type(&self) -> TypeDef {
        TypeDef::of::<T>()
    }

    fn get_arg_types(&self) -> Vec<TypeDef> {
        vec![TypeDef::of::<A1>(), TypeDef::of::<A2>()]
    }

    fn new_factory(&self, arg_getters: Vec<Box<Any>>) -> Box<Any> {
        let mut getters = arg_getters;
        box Factory::<T>::new(
            box GetterScope2::<A1, A2, T> {
                a2: getters.pop().expect("arg 2 missing").to_factory::<A2>().expect("bad arg 2"),
                a1: getters.pop().expect("arg 1 missing").to_factory::<A1>().expect("bad arg 1"),
                closure: self.clone(),
            }
            ) as Box<Any>
        }
    }

impl<'a, A1:'static, T: 'static> Getter<T> for GetterScope1<A1, T> {
    fn get(&self) -> T {
        (*(self.closure.borrow_mut().deref_mut()))(
            self.a1.get()
        )
    }

    fn boxed_clone(&self) -> Box<Getter<T> + 'static> {
        box GetterScope1::<A1, T> {
            a1: self.a1.clone(),
            closure: self.closure.clone(),
        }
    }
}

impl<'a, A1:'static, A2:'static, T: 'static> Getter<T> for GetterScope2<A1, A2, T> {
    fn get(&self) -> T {
        (*(self.closure.borrow_mut().deref_mut()))(
            self.a1.get(),
            self.a2.get()
        )
    }

    fn boxed_clone(&self) -> Box<Getter<T> + 'static> {
        box GetterScope2::<A1, A2, T> {
            a1: self.a1.clone(),
            a2: self.a2.clone(),
            closure: self.closure.clone(),
        }
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
