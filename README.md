# MetaFactory

Build value factory chains at runtime from lambdas and other sources.

[![Build Status](https://travis-ci.org/Nercury/metafactory-rs.svg?branch=master)](https://travis-ci.org/Nercury/metafactory-rs)

## Quick example

```rust
use metafactory::{ metafactory, argless_as_factory, AsFactoryExt };

fn main() {
    // initialization

    let meta_sum = metafactory(
        |a: int, b: int| a + b
    );

    let meta_twice = metafactory(
        |val: int| val * 2
    );

    // plugging in

    let any_factory = meta_twice.new(vec![
        meta_sum.new(vec![
            argless_as_factory(3i),
            argless_as_factory(2i),
        ]).ok().unwrap()
    ]).ok().unwrap();

    // using

    let getter = any_factory.as_factory_of::<int>().unwrap();

    // note that "take" requires no arguments

    assert_eq!(getter.take().value, 12);
}
```

Offers primitive reflection for returned factory type and arguments:

```rust
let meta_int = metafactory(
    |arg: bool| i32
);

// it knows the source returns 32-bit int
assert!(meta_int.get_type().is::<i32>());

// it knows the argument is bool
assert!(meta_int.get_arg_types().get(0).unwrap().is::<bool>());
```

- [Browse complete documentation for in-depth explanation and more examples](http://nercury.github.io/metafactory-rs)

## Usage

Put this in your `Cargo.toml`:

```toml
[dependencies]
metafactory = "*"
```

And this in your crate root:

```rust
extern crate metafactory;
```

## Resources

- [Full `MetaFactory` documentation](http://nercury.github.io/metafactory-rs)
- [`MetaFactory` on crates.io](https://crates.io/crates/metafactory)

## License

MIT
