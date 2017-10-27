#[macro_use]
extern crate quickcheck_derive;
extern crate quickcheck;
extern crate rand;

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

#[test]
fn unit_struct() {
    let ref mut gen = gen();
    assert_eq!(UnitStruct::arbitrary(gen), UnitStruct);
}

#[test]
fn struct_struct() {
    let ref mut gen = gen();
    assert_eq!(StructStruct::arbitrary(gen), StructStruct {
        a: -2,
        b: "ẩ".into(),
    });
}

#[test]
fn tuple_struct() {
    let ref mut gen = gen();
    assert_eq!(TupleStruct::arbitrary(gen), TupleStruct(
        -2,
        "ẩ".into(),
    ));
}

#[test]
fn generic_struct() {
    let ref mut gen = gen();
    assert_eq!(GenericStruct::<i32,String>::arbitrary(gen), GenericStruct {
        t: -2,
        u: "ẩ".into(),
    });
}

#[test]
fn big_struct() {
    let ref mut gen = gen();
    assert_eq!(BigStruct::arbitrary(gen), BigStruct {
        a: 3,
        b: UnitStruct,
        c: true,
        d: (-2, -3, -2),
        e: vec![1],
        f: StructStruct { a: -3, b: "\u{80}\u{f}".into() },
        g: (2,),
        h: -3,
        i: GenericStruct { t: (), u: "뗸".into() },
        j: Some(-3),
        k: Err(vec![()]),
        l: TupleStruct(-4, "".into()),
    });
}

fn gen() -> StdGen<IsaacRng> {
    let max_size = 4;
    StdGen::new(IsaacRng::new_unseeded(), max_size)
}
