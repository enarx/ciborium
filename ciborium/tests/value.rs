use ciborium::ser::into_writer;
use ciborium::value::Value;
use ciborium_ll::simple::{FALSE, NULL, TRUE, UNDEFINED};
use rstest::{fixture, rstest};
use serde::Serialize;
use std::collections::HashMap;

#[derive(Serialize)]
struct ExampleStruct {
    a: Vec<String>,
    b: f64,
    c: HashMap<u8, Value>,
}

#[fixture]
fn struct_instance_1() -> ExampleStruct {
    let mut map = HashMap::new();
    map.insert(15, Value::Bool(false));
    ExampleStruct {
        a: vec!["Hello".to_string(), "lol".to_string()],
        b: 3.14,
        c: map,
    }
}

#[fixture]
fn struct_instance_2() -> ExampleStruct {
    let mut map = HashMap::new();
    map.insert(15, Value::Bool(false));
    ExampleStruct {
        a: vec!["Hello".to_string()],
        b: 4.2,
        c: map,
    }
}

#[rstest(
    item,
    bytes,
    case(Value::Simple(0), "E0"),
    case(Value::Simple(1), "E1"),
    case(Value::Simple(2), "E2"),
    case(Value::Simple(3), "E3"),
    case(Value::Simple(4), "E4"),
    case(Value::Simple(5), "E5"),
    case(Value::Simple(6), "E6"),
    case(Value::Simple(7), "E7"),
    case(Value::Simple(8), "E8"),
    case(Value::Simple(9), "E9"),
    case(Value::Simple(10), "EA"),
    case(Value::Simple(11), "EB"),
    case(Value::Simple(12), "EC"),
    case(Value::Simple(13), "ED"),
    case(Value::Simple(14), "EE"),
    case(Value::Simple(15), "EF"),
    case(Value::Simple(16), "F0"),
    case(Value::Simple(17), "F1"),
    case(Value::Simple(18), "F2"),
    case(Value::Simple(19), "F3"),
    case(Value::Bool(false), "F4"),
    case(Value::Simple(FALSE), "F4"),
    case(Value::Bool(true), "F5"),
    case(Value::Simple(TRUE), "F5"),
    case(Value::Null, "F6"),
    case(Value::Simple(NULL), "F6"),
    case(Value::Simple(UNDEFINED), "F7"),
    case(Value::Simple(32), "F820"),
    case(Value::Simple(33), "F821"),
    case(Value::Simple(127), "F87F"),
    case(Value::Simple(128), "F880"),
    case(Value::Simple(255), "F8FF")
)]
fn serialize(item: Value, bytes: &str) {
    let bytes = hex::decode(bytes).unwrap();

    let mut writer: Vec<u8> = Vec::new();
    into_writer(&item, &mut writer).unwrap();

    assert_eq!(writer, bytes);
}
