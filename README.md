# QuickCheck-derive

[![Latest Version]][crates.io]
![License]

[Latest Version]: https://img.shields.io/crates/v/quickcheck\_derive.svg
[crates.io]: https://crates.io/crates/quickcheck\_derive
[License]: https://img.shields.io/crates/l/quickcheck\_derive.svg

Implementing [`quickcheck::Arbitrary`] made easy via `#[derive(Arbitrary)]`.

[`quickcheck::Arbitrary`]: https://docs.rs/quickcheck/0.4.1/quickcheck/trait.Arbitrary.html

```toml
[dependencies]
quickcheck_derive = "0.1"
```

You may be looking for:

- [QuickCheck API documentation](https://docs.rs/quickcheck/0.4.1/)
- [QuickCheck GitHub repository](https://github.com/BurntSushi/quickcheck/)
- [Koen Claessen's original QuickCheck for Haskell](http://hackage.haskell.org/package/QuickCheck/)

Supports Rust 1.19 and newer.
Older versions may work, but are not guaranteed to.


## Usage

```rust
extern crate quickcheck;
#[macro_use]
extern crate quickcheck_derive;

#[derive(Arbitrary, Clone)]     // Uses familiar #[derive(...)] syntax
struct Data {                   // Structs, ...
    foo: i32,
    bar: String,
}

#[derive(Arbitrary, Clone)]
struct Point3I(i32, i32, i32);  // ... tuple structs ...

#[derive(Arbitrary, Clone)]
struct Visitor;                 // ... and unit structs are all supported

#[derive(Arbitrary, Clone)]
enum Choice {                   // Enums with all kinds of variants too!
    Simple,
    WithNamedOptions {
        count: usize,
        numbers: Vec<u64>,
    },
    WithAnonymousOptions(bool, Option<String>),
}

#[derive(Arbitrary, Clone)]
enum State<L, U> {              // Generics? No problem!
    Locked(L),
    Unlocked(U),
}
```


## License

QuickCheck-derive is licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE.txt](LICENSE-APACHE.txt) or
   http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT.txt](LICENSE-MIT.txt) or
   http://opensource.org/licenses/MIT)

at your option.
