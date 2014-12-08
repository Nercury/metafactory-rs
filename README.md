# TypeDef

Build value factory chains at runtime from lambdas and other sources.

[![Build Status](https://travis-ci.org/Nercury/metafactory-rs.svg?branch=master)](https://travis-ci.org/Nercury/metafactory-rs)

## Quick example

```rust
use metafactory::{ metafactory, constant };
use metafactory::AsFactoryExt;

fn main() {
    // initialization

    let meta_sum = metafactory(
        |a: int, b: int| a + b
    );

    let meta_twice = metafactory(
        |val: int| val * 2
    );

    // plugging in

    let factory = meta_twice.new(vec![
        meta_sum.new(vec![
            constant(3i),
            constant(2i),
        ]).ok().unwrap()
    ]).ok().unwrap();

    // using

    let getter = foo_factory.as_factory_of::<int>().unwrap();

    assert_eq!(getter.get().value, 12);
}
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
