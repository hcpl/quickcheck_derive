extern crate quickcheck;
#[macro_use]
extern crate quickcheck_derive;
extern crate rand;


use std::fmt;
use std::i32;

use quickcheck::{Arbitrary, StdGen};
use rand::IsaacRng;


#[derive(Arbitrary, Clone, Debug, PartialEq)]
struct UnitStruct;

#[derive(Arbitrary, Clone, Debug, PartialEq)]
struct StructStruct {
    a: i32,
    b: String,
}

#[derive(Arbitrary, Clone, Debug, PartialEq)]
struct TupleStruct(i32, String);

#[derive(Arbitrary, Clone, Debug, PartialEq)]
struct GenericStruct<T,U> {
    t: T,
    u: U,
}

#[derive(Arbitrary, Clone, Debug, PartialEq)]
struct BigStruct {
    a: u64,
    b: UnitStruct,
    c: bool,
    d: (i32, i32, i32),
    e: Vec<u8>,
    f: StructStruct,
    g: (isize,),
    h: i64,
    i: GenericStruct<(), String>,
    j: Option<i64>,
    k: Result<Option<usize>, Vec<()>>,
    l: TupleStruct,
}

#[derive(Arbitrary, Clone, Debug, PartialEq)]
enum EnumEmpty {}

#[derive(Arbitrary, Clone, Debug, PartialEq)]
enum EnumWithUnitVariant {
    UnitVariant,
    StructVariant {
        a: u64,
        b: (u8, u8, u16),
    },
    TupleVariant(i8, i16, i32, i64),
}

#[derive(Arbitrary, Clone, Debug, PartialEq)]
enum EnumWithoutUnitVariant {
    StructVariant {
        a: u64,
        b: (u8, u8, u16),
    },
    TupleVariant(i8, i16, i32, i64),
}


// Generates a random value, shrinks it and matches it against correct minimums provided as
// arguments
macro_rules! check_shrinkage {
    ($( $prop_name:ident => $($minimum_values:expr),*; )*) => {
        $(
            #[test]
            fn $prop_name() {
                let mut gen = gen();
                let generated = Arbitrary::arbitrary(&mut gen);
                let minimum = shrink_to_minimum(&generated);

                // Not using .expect() because it causes type inference problems
                if minimum.is_none() {
                    panic!("Shrinking unsuccessful with the starting value {:?}", generated);
                }

                $( if minimum == Some($minimum_values) { return; } )*

                // Output allowed values on different lines
                panic!("Result of shrinking\n{:?}\ndoesn't match with any of the allowed values:\n{}",
                    minimum.unwrap(), [$( format!("{:?}\n", $minimum_values) ),*].join(""));
            }
        )*
    };
}

check_shrinkage! {
    unit_struct => UnitStruct;
    struct_struct => StructStruct { a: 0, b: "".into() };
    tuple_struct => TupleStruct(0, "".into());
    generic_struct => GenericStruct::<i32, String> { t: 0, u: "".into() };
    big_struct => BigStruct {
        a: 0,
        b: UnitStruct,
        c: false,
        d: (0, 0, 0),
        e: vec![],
        f: StructStruct { a: 0, b: "".into() },
        g: (0,),
        h: 0,
        i: GenericStruct { t: (), u: "".into() },
        j: None,
        k: Ok(None),  // <- they differ here
        l: TupleStruct(0, "".into()),
    }, BigStruct {
        a: 0,
        b: UnitStruct,
        c: false,
        d: (0, 0, 0),
        e: vec![],
        f: StructStruct { a: 0, b: "".into() },
        g: (0,),
        h: 0,
        i: GenericStruct { t: (), u: "".into() },
        j: None,
        k: Err(vec![]),  // <- they differ here
        l: TupleStruct(0, "".into()),
    };
    enum_with_unit_variant => EnumWithUnitVariant::UnitVariant;
    enum_without_unit_variant => EnumWithoutUnitVariant::TupleVariant(0, 0, 0, 0);
}


fn shrink_to_minimum<T: Clone + fmt::Debug + Arbitrary>(value: &T) -> Option<T> {
    let mut iter = value.shrink().peekable();

    // Hit the bottom, we're good
    if iter.peek().is_none() {
        return Some(value.clone());
    }

    for shrinked in iter {
        println!("{:?}", &shrinked);  // cargo shows stdout & stderr if tests fail
        // Found minimum for `shrinked` => found minimum for `value`
        if let minimum @ Some(_) = shrink_to_minimum(&shrinked) {
            return minimum;
        }
    }

    // No minimum found :(
    return None;
}

fn gen() -> StdGen<IsaacRng> {
    let max_size = 4096;  // Let's shrink from a large state space
    StdGen::new(IsaacRng::new_unseeded(), max_size)
}
