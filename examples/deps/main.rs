#![feature(conservative_impl_trait)]
#![feature(universal_impl_trait)]

extern crate metafactory;
extern crate hlist;

//use hlist::{HList, Cons, Nil};

pub struct Partial1<FPartial1>
{
    fun: FPartial1,
}

impl<FPartial1> Partial1<FPartial1>
{
    fn with<A0, Y>(
        self,
        fun_a0: impl Fn() -> A0
    )
        -> impl Fn() -> Y
        where
            FPartial1: Fn(A0) -> Y
    {
        move || (self.fun)(fun_a0())
    }
}

pub struct Partial2<FPartial2>
{
    fun: FPartial2,
}

impl<FPartial2> Partial2<FPartial2>
{
    fn with<A0, A1, Y>(
        self,
        fun_a1: impl Fn() -> A1
    )
        -> Partial1<impl Fn(A0) -> Y>
        where
            FPartial2: Fn(A0, A1) -> Y
    {
        Partial1 {
            fun: move |a0| (self.fun)(a0, fun_a1())
        }
    }
}

fn meta_final<V>(fun: impl Fn() -> V)
                 -> impl Fn() -> V
{
    move || fun()
}

fn meta_new_2<A0, A1, Y>(fun: impl Fn(A0, A1) -> Y)
                         -> Partial2<impl Fn(A0, A1) -> Y>
{
    Partial2 { fun }
}

fn main() {
    let factory_5 = meta_final(|| 5);
    let factory_str = meta_final(|| "hello");
    let factory_sum = meta_new_2(|num: i32, s: &str| num + s.len() as i32);

    let sum = factory_sum.with(factory_str).with(factory_5);

    println!("Hello {}!", sum());
}