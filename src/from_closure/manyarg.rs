//! This implements metafactory construction from closure.
//!
//! Using some big nasty macro, it supports up to 12 arguments.
//!
//! For detailed info about error handling, look at `error` mod. The
//! example bellow simply unwraps everything.
//!
//! ```
//! use metafactory::new_metafactory;
//! use metafactory::AsFactoryExt;
//!
//! fn main() {
//!     // build a metafactory from multi-argument closure.
//!     let meta_factory = new_metafactory(
//!         |a: int, b: bool, c: &str| {
//!             format!("invoked with {}, {}, {}", a, b, c)
//!         }
//!     );
//!
//!     // create a factory instance this closure.
//!     // argument factories can be constructed from cloneable sources.
//!     let factory = meta_factory.new(vec![
//!         new_metafactory(3i).new(Vec::new()).ok().unwrap(),
//!         new_metafactory(false).new(Vec::new()).ok().unwrap(),
//!         new_metafactory("hello").new(Vec::new()).ok().unwrap(),
//!     ]).ok().unwrap().as_factory_of::<String>().unwrap();
//!
//!     // value should match what factory produced.
//!     assert_eq!("invoked with 3, false, hello", factory.get());
//! }
//! ```

use std::any::Any;
use std::rc::Rc;
use std::cell::RefCell;

use typedef::TypeDef;

use super::super::{ MetaFactory, ToMetaFactory, AsFactoryExt };
use super::super::factory::{ Factory, Getter };
use super::super::error::{ FactoryErrorKind, ArgCountMismatch };

#[macro_escape]
mod macros {
    macro_rules! assert_arg_count(
        ($expected:expr, $specified:expr)
        =>
        (
            if $expected != $specified {
                return Err(
                    FactoryErrorKind::ArgCountMismatch(
                        ArgCountMismatch::new($expected, $specified)
                    )
                )
            }
        )
    )

    macro_rules! count_exprs {
        () => (0);
        ($head:expr $(, $tail:expr)*) => (1 + count_exprs!($($tail),*));
    }

    macro_rules! many_arg_closure_impl(
        ($scopeid:ident: $($argid:ident,$argt:ty,$fieldn:ident)|+)
        =>
        (
            struct $scopeid<$($argid:'static), +, T:'static> {
                $(
                    $fieldn: Factory<$argt>,
                )+
                closure: Rc<RefCell<|$($argt), +|:'static -> T>>,
            }

            impl<$($argid:'static), +, T:'static> ToMetaFactory for (|$($argt), +|:'static -> T) {
                fn to_metafactory<'a>(self) -> Box<MetaFactory + 'a> {
                    box Rc::new(RefCell::new(self))
                }
            }

            impl<$($argid:'static), +, T:'static> MetaFactory for Rc<RefCell<|$($argt), +|:'static -> T>> {
                fn get_type(&self) -> TypeDef {
                    TypeDef::of::<T>()
                }

                fn get_arg_types(&self) -> Vec<TypeDef> {
                    vec![$(TypeDef::of::<$argt>()), +]
                }

                fn new(&self, arg_getters: Vec<Box<Any>>) -> Result<Box<Any>, FactoryErrorKind> {
                    let required_argc = count_exprs!($($argid),+);

                    assert_arg_count!(required_argc, arg_getters.len());

                    let mut getters: Vec<Box<Any>> = Vec::with_capacity(arg_getters.len());
                    let mut index_names = Vec::<uint>::new();
                    let mut index = 1;
                    for v in arg_getters.into_iter().rev() {
                        getters.push(v);
                        index_names.push(index);
                        index += 1;
                    }

                    let factory = box Factory::<T>::new(
                        box $scopeid::<$($argt), +, T> {
                            $(
                                $fieldn: getters.pop()
                                    .unwrap()
                                    .as_factory_of::<$argt>()
                                    .unwrap(),
                            )+
                            closure: self.clone(),
                        }
                    ) as Box<Any>;

                    Ok(factory)
                }
            }

            impl<'a, $($argid:'static), +, T: 'static> Getter<T> for $scopeid<$($argt), +, T> {
                fn get(&self) -> T {
                    (*(self.closure.borrow_mut().deref_mut()))(
                        $(
                            self.$fieldn.get()
                        ),+
                    )
                }

                fn boxed_clone(&self) -> Box<Getter<T> + 'static> {
                    $(
                        let $fieldn = &self.$fieldn;
                    )+
                    box $scopeid::<$($argt), +, T> {
                        $(
                            $fieldn: $fieldn.clone()
                        ),
                        +,
                        closure: self.closure.clone(),
                    }
                }
            }
        )
    )
}

many_arg_closure_impl!(
    GetterScope:
    A, A, a
)

many_arg_closure_impl!(
    GetterScope2:
    A1, A1, a1 |
    A2, A2, a2
)

many_arg_closure_impl!(
    GetterScope3:
    A1, A1, a1 |
    A2, A2, a2 |
    A3, A3, a3
)

many_arg_closure_impl!(
    GetterScope4:
    A1, A1, a1 |
    A2, A2, a2 |
    A3, A3, a3 |
    A4, A4, a4
)

many_arg_closure_impl!(
    GetterScope5:
    A1, A1, a1 |
    A2, A2, a2 |
    A3, A3, a3 |
    A4, A4, a4 |
    A5, A5, a5
)

many_arg_closure_impl!(
    GetterScope6:
    A1, A1, a1 |
    A2, A2, a2 |
    A3, A3, a3 |
    A4, A4, a4 |
    A5, A5, a5 |
    A6, A6, a6
)

many_arg_closure_impl!(
    GetterScope7:
    A1, A1, a1 |
    A2, A2, a2 |
    A3, A3, a3 |
    A4, A4, a4 |
    A5, A5, a5 |
    A6, A6, a6 |
    A7, A7, a7
)

many_arg_closure_impl!(
    GetterScope8:
    A1, A1, a1 |
    A2, A2, a2 |
    A3, A3, a3 |
    A4, A4, a4 |
    A5, A5, a5 |
    A6, A6, a6 |
    A7, A7, a7 |
    A8, A8, a8
)

many_arg_closure_impl!(
    GetterScope9:
    A1, A1, a1 |
    A2, A2, a2 |
    A3, A3, a3 |
    A4, A4, a4 |
    A5, A5, a5 |
    A6, A6, a6 |
    A7, A7, a7 |
    A8, A8, a8 |
    A9, A9, a9
)

many_arg_closure_impl!(
    GetterScope10:
    A1, A1, a1 |
    A2, A2, a2 |
    A3, A3, a3 |
    A4, A4, a4 |
    A5, A5, a5 |
    A6, A6, a6 |
    A7, A7, a7 |
    A8, A8, a8 |
    A9, A9, a9 |
    A10, A10, a10
)

many_arg_closure_impl!(
    GetterScope11:
    A1, A1, a1 |
    A2, A2, a2 |
    A3, A3, a3 |
    A4, A4, a4 |
    A5, A5, a5 |
    A6, A6, a6 |
    A7, A7, a7 |
    A8, A8, a8 |
    A9, A9, a9 |
    A10, A10, a10 |
    A11, A11, a11
)

many_arg_closure_impl!(
    GetterScope12:
    A1, A1, a1 |
    A2, A2, a2 |
    A3, A3, a3 |
    A4, A4, a4 |
    A5, A5, a5 |
    A6, A6, a6 |
    A7, A7, a7 |
    A8, A8, a8 |
    A9, A9, a9 |
    A10, A10, a10 |
    A11, A11, a11 |
    A12, A12, a12
)
