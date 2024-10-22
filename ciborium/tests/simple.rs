use ciborium::{from_reader, into_writer, Value};
use ciborium_ll::simple::UNDEFINED;
use rstest::rstest;
use serde::{Deserialize, Serialize};

#[rstest]
#[case("e0", Value::Simple(0))]
#[case("e1", Value::Simple(1))]
#[case("f0", Value::Simple(16))]
#[case("f1", Value::Simple(17))]
#[case("f2", Value::Simple(18))]
#[case("f3", Value::Simple(19))]
#[case("f4", Value::Bool(false))]
#[case("f5", Value::Bool(true))]
#[case("f6", Value::Null)]
#[case("f7", Value::Simple(UNDEFINED))]
#[case("f8ff", Value::Simple(255))]
#[case("f90000", Value::Float(0f64))]
#[case("f93c00", Value::Float(1f64))]
#[case("fa00000000", Value::Float(0f64))]
#[case("fa3f800000", Value::Float(1f64))]
fn deserialize_value(#[case] bytes: &str, #[case] expected: Value) {
    let bytes = hex::decode(bytes).unwrap();
    let actual: Value = from_reader(&bytes[..]).unwrap();
    assert_eq!(actual, expected);
}

#[rstest]
#[case(Value::Simple(0), "e0")]
#[case(Value::Simple(1), "e1")]
#[case(Value::Simple(16), "f0")]
#[case(Value::Simple(17), "f1")]
#[case(Value::Simple(18), "f2")]
#[case(Value::Simple(19), "f3")]
#[case(Value::Bool(false), "f4")]
#[case(Value::Bool(true), "f5")]
#[case(Value::Null, "f6")]
#[case(Value::Simple(UNDEFINED), "f7")]
#[case(Value::Simple(255), "f8ff")]
#[case(Value::Float(0f64), "f90000")]
#[case(Value::Float(1f64), "f93c00")]
fn serialize_value(#[case] value: Value, #[case] bytes: &str) {
    let expected = hex::decode(bytes).unwrap();
    let mut actual = Vec::<u8>::new();
    into_writer(&value, &mut actual).unwrap();
    assert_eq!(actual, expected);
}

#[test]
fn do_not_accept_simple_for_integer() {
    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    pub struct NotSimple(u8);

    let bytes: [u8; 1] = [0xE0];
    let my_simple: Result<NotSimple, _> = from_reader(&bytes[..]);

    assert!(my_simple.is_err());
}
