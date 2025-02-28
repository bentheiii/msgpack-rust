use serde::Deserialize;
use serde_bytes::ByteBuf;
use std::collections::{BTreeMap, HashMap};

use rmpv::decode;
use rmpv::ext::from_value;
use rmpv::Value;

/// Tests that a `Value` is properly decoded from bytes using two different mechanisms: direct
/// deserialization using `rmp::decode::read_value` and using `serde`.
fn test_decode(buf: &[u8], v: Value) {
    let val0: Value = decode::read_value(&mut &buf[..]).unwrap();
    assert_eq!(v, val0);

    let val1: Value = rmp_serde::from_slice(buf).unwrap();
    assert_eq!(v, val1);
}

#[test]
fn test_stack_depth_checking() {
    std::thread::Builder::new()
        .name("test_stack_depth_checking".into())
        .stack_size(10_000_000)
        .spawn(|| {
            let mut buf: Vec<u8> = (0..decode::MAX_DEPTH).map(|_| 0x91).collect();
            buf.push(0xc3);

            match decode::read_value(&mut &buf[..]) {
                Ok(_) => panic!("expected max stack depth to be exceeded"),
                Err(decode::Error::DepthLimitExceeded) => {},
                Err(err) => panic!("unexpected error: {}", err),
            }
        })
        .unwrap()
        .join()
        .unwrap();
}

#[test]
fn pass_null() {
    test_decode(&[0xc0], Value::Nil);
}

#[test]
fn pass_bool() {
    test_decode(&[0xc3], Value::Boolean(true));
    test_decode(&[0xc2], Value::Boolean(false));
}

#[test]
fn pass_uint() {
    test_decode(&[0x00], Value::from(u8::min_value()));
    test_decode(&[0xcc, 0xff], Value::from(u8::MAX));
    test_decode(&[0xcd, 0xff, 0xff], Value::from(u16::MAX));
    test_decode(&[0xce, 0xff, 0xff, 0xff, 0xff], Value::from(u32::MAX));
    test_decode(&[0xcf, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff], Value::from(u64::MAX));
}

#[test]
fn pass_sint() {
    test_decode(&[0xd0, 0x80], Value::from(i8::min_value()));
    test_decode(&[0x7f], Value::from(i8::MAX));
    test_decode(&[0xd1, 0x80, 0x00], Value::from(i16::min_value()));
    test_decode(&[0xcd, 0x7f, 0xff], Value::from(i16::MAX));
    test_decode(&[0xd2, 0x80, 0x00, 0x00, 0x00], Value::from(i32::min_value()));
    test_decode(&[0xce, 0x7f, 0xff, 0xff, 0xff], Value::from(i32::MAX));
    test_decode(&[0xd3, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00], Value::from(i64::min_value()));
    test_decode(&[0xcf, 0x7f, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff], Value::from(i64::MAX));
}

#[test]
fn pass_f32() {
    test_decode(&[0xca, 0x7f, 0x7f, 0xff, 0xff], Value::from(3.4028234e38f32));
}

#[test]
fn pass_f64() {
    test_decode(&[0xcb, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00], Value::from(0.00));
    test_decode(&[0xcb, 0x40, 0x45, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00], Value::from(42.0));
}

#[test]
fn pass_str() {
    test_decode(&[0xaa, 0x6c, 0x65, 0x20, 0x6d, 0x65, 0x73, 0x73, 0x61, 0x67, 0x65],
        Value::from("le message"));
}

#[test]
fn pass_bin() {
    test_decode(&[0xc4, 0x02, 0xcc, 0x80], Value::from(&[0xcc, 0x80][..]));
}

#[test]
fn pass_array() {
    test_decode(
        &[0x92, 0xa2, 0x6c, 0x65, 0xa4, 0x73, 0x68, 0x69, 0x74],
        Value::Array(vec![Value::from("le"), Value::from("shit")]),
    );
}

#[test]
fn pass_value_map() {
    let val = Value::Map(vec![
        (Value::from(0), Value::from("le")),
        (Value::from(1), Value::from("shit")),
    ]);

    test_decode(&[0x82, 0x00, 0xa2, 0x6c, 0x65, 0x01, 0xa4, 0x73, 0x68, 0x69, 0x74], val);
}

#[test]
fn pass_uint_from_value() {
    assert_eq!(i8::min_value(), from_value(Value::from(i8::min_value())).unwrap());
    assert_eq!(i8::MAX, from_value(Value::from(i8::MAX)).unwrap());
    assert_eq!(i16::min_value(), from_value(Value::from(i16::min_value())).unwrap());
    assert_eq!(i16::MAX, from_value(Value::from(i16::MAX)).unwrap());
    assert_eq!(i32::min_value(), from_value(Value::from(i32::min_value())).unwrap());
    assert_eq!(i32::MAX, from_value(Value::from(i32::MAX)).unwrap());
    assert_eq!(i64::min_value(), from_value(Value::from(i64::min_value())).unwrap());
    assert_eq!(i64::MAX, from_value(Value::from(i64::MAX)).unwrap());
}

#[test]
fn pass_sint_from_value() {
    assert_eq!(0, from_value(Value::from(0)).unwrap());
    assert_eq!(u8::MAX, from_value(Value::from(u8::MAX)).unwrap());
    assert_eq!(u16::MAX, from_value(Value::from(u16::MAX)).unwrap());
    assert_eq!(u32::MAX, from_value(Value::from(u32::MAX)).unwrap());
    assert_eq!(u64::MAX, from_value(Value::from(u64::MAX)).unwrap());
}

#[test]
fn pass_f32_from_value() {
    assert_eq!(0.0f32, from_value(Value::from(0.0f32)).unwrap());
    assert_eq!(std::f32::consts::PI, from_value(Value::from(std::f32::consts::PI)).unwrap());
}

#[test]
fn pass_f64_from_value() {
    assert_eq!(0.0, from_value(Value::from(0.0)).unwrap());
    assert_eq!(std::f64::consts::PI, from_value(Value::from(std::f64::consts::PI)).unwrap());
}

#[test]
fn pass_char_from_value() {
    assert_eq!('c', from_value(Value::from("c")).unwrap());
}

#[test]
fn pass_str_from_value() {
    let v: String = from_value(Value::from("le message")).unwrap();
    assert_eq!("le message".to_string(), v);
}

#[test]
fn pass_bin_from_value() {
    assert_eq!(
        ByteBuf::from(&[0, 1, 2][..]),
        from_value::<ByteBuf>(Value::from(vec![0, 1, 2])).unwrap()
    );
}

#[test]
fn pass_vec_from_value() {
    let v: Vec<String> = from_value(Value::from(vec![Value::from("John"), Value::from("Smith")])).unwrap();
    assert_eq!(vec!["John".to_string(), "Smith".to_string()], v);
}

#[test]
fn pass_map_from_value() {
    let mut map = BTreeMap::new();
    map.insert("name".to_string(), "John".to_string());
    map.insert("surname".to_string(), "Smith".to_string());

    let val = Value::from(vec![
        (Value::from("name"), Value::from("John")),
        (Value::from("surname"), Value::from("Smith")),
    ]);

    let v: BTreeMap<String, String> = from_value(val).unwrap();

    assert_eq!(map, v);
}

#[test]
fn pass_option_from_value() {
    assert_eq!(None::<i32>, from_value(Value::Nil).unwrap());
    // TODO: assert_eq!(Some(None::<i32>), from_value(Value::Nil).unwrap());
    assert_eq!(Some(42), from_value(Value::from(42)).unwrap());
    assert_eq!(Some(Some(42)), from_value(Value::from(42)).unwrap());
}

#[test]
fn pass_seq_from_value() {
    let v: Vec<u64> = from_value(Value::Array(vec![Value::from(0), Value::from(42)])).unwrap();
    assert_eq!(vec![0, 42], v);
}

#[test]
fn pass_tuple_from_value() {
    let v: (String, u8) = from_value(Value::Array(vec![Value::from("John"), Value::from(42)])).unwrap();
    assert_eq!(("John".into(), 42), v);
}

#[test]
fn pass_unit_struct_from_value() {
    #[derive(Debug, PartialEq, Deserialize)]
    struct Unit;

    assert_eq!(Unit, from_value(Value::Array(vec![])).unwrap());
}

#[test]
fn pass_newtype_struct_from_value() {
    #[derive(Debug, PartialEq, Deserialize)]
    struct Newtype(String);

    assert_eq!(
        Newtype("John".into()),
        from_value(Value::from("John")).unwrap()
    );
}

#[test]
fn pass_tuple_struct_from_value() {
    #[derive(Debug, PartialEq, Deserialize)]
    struct Newtype(String, u8);

    assert_eq!(
        Newtype("John".into(), 42),
        from_value(Value::Array(vec![Value::from("John"), Value::from(42)])).unwrap()
    );
}

#[test]
fn pass_struct_from_value() {
    #[derive(Debug, PartialEq, Deserialize)]
    struct Struct {
        name: String,
        age: u8,
    }

    assert_eq!(Struct { name: "John".into(), age: 42 },
        from_value(Value::Array(vec![Value::from("John"), Value::from(42)])).unwrap());
}

#[test]
fn pass_enum_from_value() {
    #[derive(Debug, PartialEq, Deserialize)]
    enum Enum {
        Unit,
        Newtype(String),
        Tuple(String, u32),
        Struct { name: String, age: u32 },
    }

    assert_eq!(Enum::Unit,
        from_value(Value::Array(vec![Value::from(0), Value::Array(vec![])])).unwrap());
    assert_eq!(Enum::Newtype("John".into()),
        from_value(Value::Array(vec![Value::from(1), Value::Array(vec![Value::from("John")])])).unwrap());
    assert_eq!(Enum::Tuple("John".into(), 42),
        from_value(Value::Array(vec![Value::from(2), Value::Array(vec![Value::from("John"), Value::from(42)])])).unwrap());
    assert_eq!(Enum::Struct { name: "John".into(), age: 42 },
        from_value(Value::Array(vec![Value::from(3), Value::Array(vec![Value::from("John"), Value::from(42)])])).unwrap());
}

#[test]
fn pass_tuple_struct_from_ext() {
    #[derive(Debug, PartialEq)]
    struct ExtStruct(i8, Vec<u8>);

    struct ExtStructVisitor;

    impl<'de> serde::de::Visitor<'de> for ExtStructVisitor {
        type Value = ExtStruct;

        fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            formatter.write_str("msgpack ext")
        }

        fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
            where D: serde::de::Deserializer<'de>,
        {
            deserializer.deserialize_tuple(2, self)
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where A: serde::de::SeqAccess<'de>,
        {

            let tag = seq.next_element()?
                .ok_or_else(|| serde::de::Error::invalid_length(0, &self))?;
            let bytes: serde_bytes::ByteBuf = seq.next_element()?
                .ok_or_else(|| serde::de::Error::invalid_length(1, &self))?;

            Ok(ExtStruct(tag, bytes.to_vec()))
        }
    }

    impl<'de> serde::de::Deserialize<'de> for ExtStruct {
        fn deserialize<D>(deserializer: D) -> Result<ExtStruct, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_any(ExtStructVisitor)
        }
    }

    assert_eq!(
        ExtStruct(42, vec![255]),
        from_value(Value::Ext(42, vec![255])).unwrap()
    );
}

#[derive(Debug, PartialEq)]
enum MightFail<T>{
    Ok(T),
    Failed,
}

impl<'de, T:serde::de::Deserialize<'de>> serde::de::Deserialize<'de> for MightFail<T> {
    fn deserialize<D>(deserializer: D) -> Result<MightFail<T>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        match T::deserialize(deserializer){
            Ok(v) => Ok(MightFail::Ok(v)),
            Err(_) => Ok(MightFail::Failed),
        }
    }
}

#[test]
fn pass_failing_elements() {
    let buffer = rmp_serde::to_vec(&(42,
        41,
        "hi there",
        43,
        (1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16),
        4.52,
        4,
        1u64 << 63,
        "hi", // test fixed string
        65,
        (1,2,3), // test fixed arrays
        &[0xcc, 0x80][..], // test bin
        HashMap::from([("a", 1), ("b", 2), ("c", 3)]), // test fixed map
        HashMap::from([("a", 1), ("b", 2), ("c", 3), ("d", 4), ("e", 5), ("f", 6), ("g", 7), ("h", 8), ("i", 9), ("j", 10), ("k", 11), ("l", 12), ("m", 13), ("n", 14), ("o", 15), ("p", 16)]), // test map
        66,
    )).unwrap();
    let deserialized: Vec<MightFail<i32>> = rmp_serde::from_slice(&buffer).unwrap();
    assert_eq!(deserialized, vec![
        MightFail::Ok(42),
        MightFail::Ok(41),
        MightFail::Failed,
        MightFail::Ok(43),
        MightFail::Failed,
        MightFail::Failed,
        MightFail::Ok(4),
        MightFail::Failed,
        MightFail::Failed,
        MightFail::Ok(65),
        MightFail::Failed,
        MightFail::Failed,
        MightFail::Failed,
        MightFail::Failed,
        MightFail::Ok(66),
    ]);
}