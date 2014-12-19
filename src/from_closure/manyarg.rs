//! This implements metafactory construction from a closure.
//!
//! Using some macro magic, it supports up to 12 arguments.
//!
//! ```
//! use metafactory::metafactory;
//! use metafactory::AsFactoryExt;
//!
//! fn main() {
//!     // build a metafactory from multi-argument closure.
//!     let meta_factory = metafactory(
//!         |a: int, b: bool, c: &'static str| {
//!             format!("invoked with {}, {}, {}", a, b, c)
//!         }
//!     );
//!
//!     // create a factory instance this closure.
//!     // argument factories can be constructed from cloneable sources.
//!     let factory = meta_factory.new(vec![
//!         metafactory(3i).new(Vec::new()).ok().unwrap(),
//!         metafactory(false).new(Vec::new()).ok().unwrap(),
//!         metafactory("hello").new(Vec::new()).ok().unwrap(),
//!     ]).ok().unwrap().as_factory_of::<String>().unwrap();
//!
//!     // value should match what factory produced.
//!     assert_eq!("invoked with 3, false, hello", factory.take());
//! }
//! ```

use std::any::Any;
use std::rc::Rc;
use std::cell::RefCell;

use typedef::TypeDef;

use super::super::{ MetaFactory, ToMetaFactory, AsFactoryExt };
use super::super::factory::{ Factory, Getter };
use super::super::error::{ FactoryErrorKind, ArgCountMismatch, ArgTypeMismatch };

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
    );

    macro_rules! try_unwrap_factory(
        ($T:ty, $factory:expr, $index:ident)
        =>
        (
            match $factory.as_factory_of::<$T>() {
                Some(factory) => factory,
                None => {
                    return Err(
                        FactoryErrorKind::ArgTypeMismatch(
                            ArgTypeMismatch::new(TypeDef::of::<$T>(), $index)
                        )
                    );
                }
            }
        )
    );

    macro_rules! count_exprs {
        () => (0);
        ($head:expr $(, $tail:expr)*) => (1 + count_exprs!($($tail),*));
    }

    macro_rules! many_arg_closure_impl(
        ($GetterScope:ident: $($_A:ident,$_AT:ty,$_a:ident)|+)
        =>
        (
            /// Contains references to parent factories, so they
            /// can be invoked directly to get arguments for the closure.
            struct $GetterScope<$($_A:'static), +, T:'static> {
                // References to argument factories.
                $(
                    $_a: Factory<$_AT>,
                )+
                // Closure reference.
                closure: Rc<RefCell<|$($_AT), +|:'static -> T>>,
            }

            /// Implement `ToMetaFactory` conversion for closures
            /// |A1, A2, ... AN| -> T
            impl<$($_A:'static), +, T:'static> ToMetaFactory for (|$($_AT), +|:'static -> T) {
                fn to_metafactory<'a>(self) -> Box<MetaFactory + 'a> {
                    box Rc::new(RefCell::new(self))
                }
            }

            /// Use the closure reference itself as metafactory. For now I did
            /// not notice any issues with this, but the Rc can be put into some
            /// wrapper struct, and then we could implement MetaFactory for that.
            impl<$($_A:'static), +, T:'static> MetaFactory for Rc<RefCell<|$($_AT), +|:'static -> T>> {
                fn get_type(&self) -> TypeDef {
                    TypeDef::of::<T>()
                }

                fn get_arg_types(&self) -> Vec<TypeDef> {
                    // Just go over the types and output TypeDefs for them.
                    vec![$(TypeDef::of::<$_AT>()), +]
                }

                #[allow(unused_assignments)]
                fn new(&self, arg_getters: Vec<Box<Any>>) -> Result<Box<Any>, FactoryErrorKind> {
                    // Calculate required argument count from specified type count.
                    let required_argc = count_exprs!($($_A),+);

                    // Return error if count does not match.
                    assert_arg_count!(required_argc, arg_getters.len());

                    let mut getters = arg_getters;
                    let mut arg_index = 0;
                    $(
                        let $_a; // Factory object of correct type.
                        { // Scope so we can reuse `maybe_factory`.
                            // Already checked the argc above, so unwrap.
                            let maybe_factory = getters.remove(0).unwrap();

                            // Return error if factory does not have a correct type.
                            $_a = try_unwrap_factory!($_AT, maybe_factory, arg_index);

                            arg_index += 1;
                        }
                    )+

                    // Build factory instance for THIS closure copy, passing
                    // all parent factories as arguments.
                    let factory = box Factory::<T>::new(
                        box $GetterScope::<$($_AT), +, T> {
                            $(
                                $_a: $_a,
                            )+
                            closure: self.clone(),
                        }
                    ) as Box<Any>;

                    Ok(factory)
                }
            }

            /// Use GetterScope as a value getter. This is part
            /// that is actually used at runtime, and would benefit from
            /// any further optimizations.
            impl<'a, $($_A:'static), +, T: 'static> Getter<T> for $GetterScope<$($_AT), +, T> {
                fn take(&self) -> T {
                    (*(self.closure.borrow_mut().deref_mut()))(
                        $(
                            self.$_a.take()
                        ),+
                    )
                }

                fn boxed_clone(&self) -> Box<Getter<T> + 'static> {
                    $(
                        let $_a = &self.$_a;
                    )+
                    box $GetterScope::<$($_AT), +, T> {
                        $(
                            $_a: $_a.clone()
                        ),
                        +,
                        closure: self.closure.clone(),
                    }
                }
            }
        )
    );
}

many_arg_closure_impl!(
    GetterScope:
    A, A, a
);

many_arg_closure_impl!(
    GetterScope2:
    A1, A1, a1 |
    A2, A2, a2
);

many_arg_closure_impl!(
    GetterScope3:
    A1, A1, a1 |
    A2, A2, a2 |
    A3, A3, a3
);

many_arg_closure_impl!(
    GetterScope4:
    A1, A1, a1 |
    A2, A2, a2 |
    A3, A3, a3 |
    A4, A4, a4
);

many_arg_closure_impl!(
    GetterScope5:
    A1, A1, a1 |
    A2, A2, a2 |
    A3, A3, a3 |
    A4, A4, a4 |
    A5, A5, a5
);

many_arg_closure_impl!(
    GetterScope6:
    A1, A1, a1 |
    A2, A2, a2 |
    A3, A3, a3 |
    A4, A4, a4 |
    A5, A5, a5 |
    A6, A6, a6
);

many_arg_closure_impl!(
    GetterScope7:
    A1, A1, a1 |
    A2, A2, a2 |
    A3, A3, a3 |
    A4, A4, a4 |
    A5, A5, a5 |
    A6, A6, a6 |
    A7, A7, a7
);

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
);

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
);

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
);

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
);

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
);

#[cfg(test)]
mod test {
    use std::any::Any;
    use typedef::TypeDef;
    use super::super::super::{ ToMetaFactory, MetaFactory, AsFactoryExt }; // super
    use super::super::super::error::{ FactoryErrorKind }; // really super

    #[test]
    fn should_work_with_1_arg_closure() {
        assert_eq!(
            create(
                |ok: bool| {
                    if ok { 5i } else { 6i }
                },
                vec![
                    arg(false)
                ]
            ).as_factory_of::<int>().unwrap().take(),
            6i
        );
    }

    #[test]
    fn should_work_with_2_arg_closure() {
        assert_eq!(
            create(
                |a: u8, b: u8| {
                    (a + b) as uint
                },
                vec![
                    arg(13u8), arg(12u8)
                ]
            ).as_factory_of::<uint>().unwrap().take(),
            13u + 12u
        );
    }

    #[test]
    fn should_return_arg_count_mismatch_for_3_arg_closure_with_2_args() {
        match maybe_create(
            |a: i8, b: i8, c:i8| {
                a + b + c
            },
            vec![
                arg(1i8), arg(1i8)
            ]
        ) {
            Err(FactoryErrorKind::ArgCountMismatch(e)) => {
                assert_eq!(e.expected, 3);
                assert_eq!(e.specified, 2);
            },
            _ => panic!("Expected ArgCountMismatch error!"),
        }
    }

    #[test]
    fn should_return_arg_type_mismatch_for_bad_arg_0() {
        match maybe_create(
            |a: i8, b: i8, c:i8| {
                a + b + c
            },
            vec![
                arg(true), arg(1i8), arg(1i8)
            ]
        ) {
            Err(FactoryErrorKind::ArgTypeMismatch(e)) => {
                assert_eq!(e.expected_type, TypeDef::of::<i8>());
                assert_eq!(e.argument_index, 0);
            },
            _ => panic!("Expected ArgTypeMismatch error!"),
        }
    }

    #[test]
    fn should_return_arg_type_mismatch_for_bad_arg_1() {
        match maybe_create(
            |a: i8, b: i8, c:i8| {
                a + b + c
            },
            vec![
                arg(1i8), arg("hello :)"), arg(1i8)
            ]
        ) {
            Err(FactoryErrorKind::ArgTypeMismatch(e)) => {
                assert_eq!(e.expected_type, TypeDef::of::<i8>());
                assert_eq!(e.argument_index, 1);
            },
            _ => panic!("Expected ArgTypeMismatch error!"),
        }
    }

    #[test]
    fn should_return_arg_type_mismatch_for_bad_arg_2() {
        match maybe_create(
            |a: i8, b: i8, c:i8| {
                a + b + c
            },
            vec![
            arg(1i8), arg(1i8), arg(box 23.3f64)
            ]
        ) {
            Err(FactoryErrorKind::ArgTypeMismatch(e)) => {
                assert_eq!(e.expected_type, TypeDef::of::<i8>());
                assert_eq!(e.argument_index, 2);
            },
            _ => panic!("Expected ArgTypeMismatch error!"),
        }
    }

    #[test]
    fn should_return_arg_type_mismatch_for_bad_arg_1_and_2() {
        match maybe_create(
            |a: i8, b: i8, c:i8| {
                a + b + c
            },
            vec![
            arg(1i8), arg(false), arg(box 23.3f64)
            ]
        ) {
            Err(FactoryErrorKind::ArgTypeMismatch(e)) => {
                assert_eq!(e.expected_type, TypeDef::of::<i8>());
                assert_eq!(e.argument_index, 1);
            },
            _ => panic!("Expected ArgTypeMismatch error!"),
        }
    }

    #[test]
    fn should_work_with_12_arg_closure() {
        assert_eq!(
            create(
                |a1: i16, a2: i16, a3: i16, a4: i16, a5: i16, a6: i16,
                 a7: i16, a8: i16, a9: i16, a10: i16, a11: i16, a12: i16| {
                    a1 + a2 + a3 + a4 + a5 + a6 + a7
                    + a8 + a9 + a10 + a11 + a12
                },
                vec![
                    arg(3i16), arg(3i16), arg(3i16), arg(3i16), arg(3i16),
                    arg(3i16), arg(3i16), arg(3i16), arg(3i16), arg(3i16),
                    arg(3i16), arg(3i16)
                ]
            ).as_factory_of::<i16>().unwrap().take(),
            3i16 * 12
        );
    }

    fn create<T: ToMetaFactory>(source: T, args: Vec<Box<Any>>) -> Box<Any> {
        source.to_metafactory().new(args).ok().unwrap()
    }

    fn maybe_create<T: ToMetaFactory>(source: T, args: Vec<Box<Any>>) -> Result<Box<Any>, FactoryErrorKind>  {
        source.to_metafactory().new(args)
    }

    fn arg<T: ToMetaFactory>(source: T) -> Box<Any> {
        source.to_metafactory().new(Vec::new()).ok().unwrap()
    }
}
