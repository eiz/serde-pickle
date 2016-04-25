// Copyright (c) 2015-2016 Georg Brandl.  Licensed under the Apache License,
// Version 2.0 <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0>
// or the MIT license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at
// your option. This file may not be copied, modified, or distributed except
// according to those terms.

extern crate rand;
extern crate quickcheck;
extern crate serde_json;

mod arby;

macro_rules! pyobj {
    (n=None)     => { Value::None };
    (b=True)     => { Value::Bool(true) };
    (b=False)    => { Value::Bool(false) };
    (i=$i:expr)  => { Value::I64($i) };
    (ii=$i:expr) => { Value::Int($i.clone()) };
    (f=$f:expr)  => { Value::F64($f) };
    (bb=$b:expr) => { Value::Bytes($b.to_vec()) };
    (s=$s:expr)  => { Value::String($s.into()) };
    (t=($($m:ident=$v:tt),*))  => { Value::Tuple(vec![$(pyobj!($m=$v)),*]) };
    (l=[$($m:ident=$v:tt),*])  => { Value::List(vec![$(pyobj!($m=$v)),*]) };
    (ss=($($m:ident=$v:tt),*)) => { Value::Set(BTreeSet::from_iter(vec![$(hpyobj!($m=$v)),*])) };
    (fs=($($m:ident=$v:tt),*)) => { Value::FrozenSet(BTreeSet::from_iter(vec![$(hpyobj!($m=$v)),*])) };
    (d={$($km:ident=$kv:tt => $vm:ident=$vv:tt),*}) => {
        Value::Dict(BTreeMap::from_iter(vec![$((hpyobj!($km=$kv),
                                                pyobj!($vm=$vv))),*])) };
}

macro_rules! hpyobj {
    (n=None)     => { HashableValue::None };
    (b=True)     => { HashableValue::Bool(true) };
    (b=False)    => { HashableValue::Bool(false) };
    (i=$i:expr)  => { HashableValue::I64($i) };
    (ii=$i:expr) => { HashableValue::Int($i.clone()) };
    (f=$f:expr)  => { HashableValue::F64($f) };
    (bb=$b:expr) => { HashableValue::Bytes($b.to_vec()) };
    (s=$s:expr)  => { HashableValue::String($s.into()) };
    (t=($($m:ident=$v:tt),*))  => { HashableValue::Tuple(vec![$(hpyobj!($m=$v)),*]) };
    (fs=($($m:ident=$v:tt),*)) => { HashableValue::FrozenSet(BTreeSet::from_iter(vec![$(hpyobj!($m=$v)),*])) };
}

mod struct_tests {
    use std::fmt;
    use std::iter::FromIterator;
    use std::collections::BTreeMap;
    use serde::{ser, de};
    use {to_vec, value_to_vec, from_slice, value_from_slice, to_value, from_value,
         Value, HashableValue};

    #[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
    struct Inner {
        a: (),
        b: usize,
        c: Vec<String>,
    }

    #[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
    struct Outer {
        inner: Vec<Inner>,
    }

    #[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
    struct Unit;

    #[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
    struct Newtype(i32);

    #[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
    struct Tuple(i32, bool);

    #[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
    #[serde(deny_unknown_fields)]
    enum Animal {
        Dog,
        AntHive(Vec<String>),
        Frog(String, Vec<isize>),
        Cat { age: usize, name: String },
    }

    fn test_encode_ok<T>(value: T, target: Value)
        where T: PartialEq + ser::Serialize,
    {
        // Test serialization via pickle.
        let vec = to_vec(&value, true).unwrap();
        let py_val: Value = value_from_slice(&vec).unwrap();
        assert_eq!(py_val, target);
        // Test direct serialization to Value.
        let py_val: Value = to_value(&value).unwrap();
        assert_eq!(py_val, target);
    }

    fn test_decode_ok<T>(pyvalue: Value, target: T)
        where T: PartialEq + fmt::Debug + de::Deserialize,
    {
        // Test deserialization from pickle.
        let vec = value_to_vec(&pyvalue, true).unwrap();
        let val: T = from_slice(&vec).unwrap();
        assert_eq!(val, target);
        // Test direct deserialization from Value.
        let val: T = from_value(pyvalue).unwrap();
        assert_eq!(val, target);
    }

    #[test]
    fn encode_types() {
        test_encode_ok((), pyobj!(n=None));
        test_encode_ok(true, pyobj!(b=True));
        test_encode_ok(None::<i32>, pyobj!(n=None));
        test_encode_ok(Some(false), pyobj!(b=False));
        test_encode_ok(10000000000_i64, pyobj!(i=10000000000));
        test_encode_ok(4.5_f64, pyobj!(f=4.5));
        test_encode_ok('ä', pyobj!(s="ä"));
        test_encode_ok("string", pyobj!(s="string"));
        // serde doesn't encode into bytes...
        test_encode_ok(b"\x00\x01", pyobj!(l=[i=0, i=1]));
        test_encode_ok(vec![1, 2, 3], pyobj!(l=[i=1, i=2, i=3]));
        test_encode_ok((1, 2, 3), pyobj!(t=(i=1, i=2, i=3)));
        test_encode_ok([1, 2, 3], pyobj!(l=[i=1, i=2, i=3]));
        test_encode_ok(BTreeMap::from_iter(vec![(1, 2), (3, 4)]),
                       pyobj!(d={i=1 => i=2, i=3 => i=4}));
    }

    #[test]
    fn encode_struct() {
        test_encode_ok(Unit,
                       pyobj!(t=()));
        test_encode_ok(Newtype(42),
                       pyobj!(i=42));
        test_encode_ok(Tuple(42, false),
                       pyobj!(t=(i=42, b=False)));
        test_encode_ok(Inner { a: (), b: 32, c: vec!["doc".into()] },
                       pyobj!(d={s="a" => n=None, s="b" => i=32,
                                 s="c" => l=[s="doc"]}));
    }

    #[test]
    fn encode_enum() {
        test_encode_ok(Animal::Dog,
                       pyobj!(t=(s="Dog")));
        test_encode_ok(Animal::AntHive(vec!["ant".into(), "aunt".into()]),
                       pyobj!(t=(s="AntHive", l=[s="ant", s="aunt"])));
        test_encode_ok(Animal::Frog("Henry".into(), vec![1, 5]),
                       pyobj!(t=(s="Frog", l=[s="Henry", l=[i=1, i=5]])));
        test_encode_ok(Animal::Cat { age: 5, name: "Molyneux".into() },
                       pyobj!(t=(s="Cat", d={s="age" => i=5, s="name" => s="Molyneux"})));
    }

    #[test]
    fn decode_types() {
        test_decode_ok(pyobj!(n=None), ());
        test_decode_ok(pyobj!(b=True), true);
        test_decode_ok(pyobj!(b=True), Some(true));
        test_decode_ok::<Option<bool>>(pyobj!(n=None), None);
        test_decode_ok(pyobj!(i=10000000000), 10000000000_i64);
        test_decode_ok(pyobj!(f=4.5), 4.5_f64);
        test_decode_ok(pyobj!(s="ä"), 'ä');
        test_decode_ok(pyobj!(s="string"), String::from("string"));
        // Vec<u8> doesn't decode from serde bytes...
        test_decode_ok(pyobj!(bb=b"bytes"), String::from("bytes"));
        test_decode_ok(pyobj!(l=[i=1, i=2, i=3]), vec![1, 2, 3]);
        test_decode_ok(pyobj!(t=(i=1, i=2, i=3)), (1, 2, 3));
        test_decode_ok(pyobj!(l=[i=1, i=2, i=3]), [1, 2, 3]);
        test_decode_ok(pyobj!(d={i=1 => i=2, i=3 => i=4}),
                       BTreeMap::from_iter(vec![(1, 2), (3, 4)]));
    }

    #[test]
    fn decode_struct() {
        test_decode_ok(pyobj!(t=()),
                       Unit);
        test_decode_ok(pyobj!(i=42),
                       Newtype(42));
        test_decode_ok(pyobj!(t=(i=42, b=False)),
                       Tuple(42, false));
        test_decode_ok(pyobj!(d={s="a" => n=None, s="b" => i=32, s="c" => l=[s="doc"]}),
                       Inner { a: (), b: 32, c: vec!["doc".into()] });
    }

    #[test]
    fn decode_enum() {
        test_decode_ok(pyobj!(t=(s="Dog")),
                       Animal::Dog);
        test_decode_ok(pyobj!(t=(s="AntHive", l=[s="ant", s="aunt"])),
                       Animal::AntHive(vec!["ant".into(), "aunt".into()]));
        test_decode_ok(pyobj!(t=(s="Frog", l=[s="Henry", l=[i=1, i=5]])),
                       Animal::Frog("Henry".into(), vec![1, 5]));
        test_decode_ok(pyobj!(t=(s="Cat", d={s="age" => i=5, s="name" => s="Molyneux"})),
                       Animal::Cat { age: 5, name: "Molyneux".into() });
        test_decode_ok(pyobj!(l=[t=(s="Dog"), t=(s="Dog"),
                                 t=(s="Cat", d={s="age" => i=5, s="name" => s="?"})]),
                       vec![Animal::Dog, Animal::Dog, Animal::Cat { age: 5, name: "?".into() }]);
    }
}

mod value_tests {
    use std::fs::File;
    use std::collections::{BTreeMap, BTreeSet};
    use std::iter::FromIterator;
    use num_bigint::BigInt;
    use super::rand::{Rng, thread_rng};
    use super::quickcheck::{QuickCheck, StdGen};
    use super::serde_json;
    use {value_from_reader, value_to_vec, value_from_slice, to_vec, from_slice};
    use {Value, HashableValue};
    use error::{Error, ErrorCode};

    // combinations of (python major, pickle proto) to test
    const TEST_CASES: &'static [(u32, u32)] = &[
        (2, 0), (2, 1), (2, 2),
        (3, 0), (3, 1), (3, 2), (3, 3), (3, 4)
    ];

    fn get_test_object() -> Value {
        // Reproduces the test_object from test/data/generate.py.
        let longish = BigInt::from(10000000000u64) * BigInt::from(10000000000u64);
        pyobj!(d={
            n=None           => n=None,
            b=False          => t=(b=False, b=True),
            i=10             => i=100000,
            ii=longish       => ii=longish,
            f=1.0            => f=1.0,
            bb=b"bytes"      => bb=b"bytes",
            s="string"       => s="string",
            fs=(i=0, i=42)   => fs=(i=0, i=42),
            t=(i=1, i=2)     => t=(i=1, i=2, i=3),
            t=()             => l=[
                l=[i=1, i=2, i=3],
                ss=(i=0, i=42),
                d={}
            ]
        })
    }

    #[test]
    fn unpickle_all() {
        let comparison = get_test_object();

        for &(major, proto) in TEST_CASES {
            let file = File::open(format!("test/data/tests_py{}_proto{}.pickle", major, proto)).unwrap();
            let unpickled = value_from_reader(file).unwrap();
            assert_eq!(unpickled, comparison);
        }
    }

    #[test]
    fn roundtrip() {
        let dict = get_test_object();
        let vec: Vec<_> = value_to_vec(&dict, true).unwrap();
        let tripped = value_from_slice(&vec).unwrap();
        assert_eq!(dict, tripped);
    }

    #[test]
    fn recursive() {
        for proto in &[0, 1, 2, 3, 4] {
            let file = File::open(format!("test/data/test_recursive_proto{}.pickle", proto)).unwrap();
            match value_from_reader(file) {
                Err(Error::Syntax(ErrorCode::Recursive)) => { }
                _ => assert!(false, "wrong/no error returned for recursive structure")
            }
        }
    }

    #[test]
    fn fuzzing() {
        // Tries to ensure that we don't panic when encountering strange streams.
        for _ in 0..1000 {
            let mut stream = [0u8; 1000];
            thread_rng().fill_bytes(&mut stream);
            if *stream.last().unwrap() == b'.' { continue; }
            // These must all fail with an error, since we skip the check if the
            // last byte is a STOP opcode.
            assert!(value_from_slice(&stream).is_err());
        }
    }

    #[test]
    fn qc_roundtrip() {
        fn roundtrip(original: Value) {
            let vec: Vec<_> = value_to_vec(&original, true).unwrap();
            let tripped = value_from_slice(&vec).unwrap();
            assert_eq!(original, tripped);
        }
        QuickCheck::new().gen(StdGen::new(thread_rng(), 10))
                         .tests(5000)
                         .quickcheck(roundtrip as fn(_));
    }

    #[test]
    fn roundtrip_json() {
        let original: serde_json::Value = serde_json::from_str(r#"[
            {"null": null,
             "false": false,
             "true": true,
             "int": -1238571,
             "float": 1.5e10,
             "list": [false, 5, "true", 3.8]
            }
        ]"#).unwrap();
        let vec: Vec<_> = to_vec(&original, true).unwrap();
        let tripped = from_slice(&vec).unwrap();
        assert_eq!(original, tripped);
    }
}

#[cfg(test)]
mod benches {
    extern crate test;

    use std::collections::BTreeMap;
    use byteorder::{LittleEndian, WriteBytesExt};
    use self::test::Bencher;
    use {Value, HashableValue, value_from_slice, value_to_vec};

    #[bench]
    fn unpickle_list(b: &mut Bencher) {
        // Creates [[0], [1], [2], ...]
        // Start a list
        let mut buffer = b"\x80\x02]q\x00(".to_vec();
        for i in 0..1000 {
            // Insert an empty list (memoized)
            buffer.extend(b"]r");
            buffer.write_u32::<LittleEndian>(i + 1).unwrap();
            // Insert i as an integer
            buffer.push(b'M');
            buffer.write_u16::<LittleEndian>(i as u16).unwrap();
            // Append
            buffer.push(b'a');
        }
        // Append all
        buffer.extend(b"e.");
        b.iter(|| value_from_slice(&buffer).unwrap());
    }

    #[bench]
    fn unpickle_list_no_memo(b: &mut Bencher) {
        // Same as above, but doesn't use the memo
        let mut buffer = b"\x80\x02](".to_vec();
        for i in 0..1000 {
            buffer.extend(b"]M");
            buffer.write_u16::<LittleEndian>(i as u16).unwrap();
            buffer.push(b'a');
        }
        buffer.extend(b"e.");
        b.iter(|| value_from_slice(&buffer).unwrap());
    }

    #[bench]
    fn unpickle_dict(b: &mut Bencher) {
        // Creates {0: "string", 1: "string", ...}
        let mut buffer = b"\x80\x03}q\x00(K\x00".to_vec();
        buffer.extend(b"X\x06\x00\x00\x00stringq\x01");
        for i in 0..1000 {
            buffer.push(b'M');
            buffer.write_u16::<LittleEndian>(i as u16).unwrap();
            buffer.extend(b"h\x01");
        }
        buffer.extend(b"u.");
        b.iter(|| value_from_slice(&buffer).unwrap());
    }

    #[bench]
    fn unpickle_nested_list(b: &mut Bencher) {
        // Creates [[[[...]]]]
        let mut buffer = b"\x80\x02".to_vec();
        for i in 0..201 {
            buffer.extend(b"]r");
            buffer.write_u32::<LittleEndian>(i).unwrap();
        }
        for _ in 0..200 {
            buffer.push(b'a');
        }
        buffer.push(b'.');
        b.iter(|| value_from_slice(&buffer).unwrap());
    }

    #[bench]
    fn unpickle_nested_list_no_memo(b: &mut Bencher) {
        // Creates [[[[...]]]] without using memo
        let mut buffer = b"\x80\x02".to_vec();
        for _ in 0..201 {
            buffer.extend(b"]");
        }
        for _ in 0..200 {
            buffer.push(b'a');
        }
        buffer.push(b'.');
        b.iter(|| value_from_slice(&buffer).unwrap());
    }

    #[bench]
    fn unpickle_simple_tuple(b: &mut Bencher) {
        let mut list = Vec::with_capacity(1000);
        for i in 0..1000 {
            list.push(pyobj!(i=i));
        }
        let tuple = Value::Tuple(list);
        let buffer = value_to_vec(&tuple, true).unwrap();
        b.iter(|| value_from_slice(&buffer).unwrap());
    }

    #[bench]
    fn pickle_list(b: &mut Bencher) {
        let mut list = Vec::with_capacity(1000);
        for i in 0..1000 {
            list.push(pyobj!(l=[i=i]));
        }
        let list = Value::List(list);
        b.iter(|| value_to_vec(&list, true).unwrap());
    }

    #[bench]
    fn pickle_dict(b: &mut Bencher) {
        let mut dict = BTreeMap::new();
        for i in 0..1000 {
            dict.insert(hpyobj!(i=i), pyobj!(l=[i=i]));
        }
        let dict = Value::Dict(dict);
        b.iter(|| value_to_vec(&dict, true).unwrap());
    }
}
