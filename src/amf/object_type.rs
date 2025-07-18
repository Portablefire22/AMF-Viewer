use std::collections::HashMap;
use std::fmt;
use std::fmt::Formatter;

#[derive(Clone, Debug, PartialEq)]
pub enum ObjectType {
    Amf0Number(f64),
    Amf0Bool(bool),
    Amf0String(String),
    Amf0Object,
    Amf0Null,
    Amf0Undefined,
    Amf0EcmaArray,
    Amf0StrictArray,
    Amf0Date,
    Amf0LongString,
    Amf0XML,
    Amf0TypedObject,
    Amf0Switch,

    Amf3Undefined,
    Amf3Null,
    Amf3False,
    Amf3True,
    Amf3Integer(i32),
    Amf3Double(f64),
    Amf3String(String),
    Amf3XMLDocument,
    Amf3Date,
    Amf3Array(Vec<isize>),
    Amf3Object(HashMap<String, Option<isize>>),
    Amf3XML,
    Amf3ByteArray,
    Amf3VectorInt,
    Amf3VectorUInt,
    Amf3VectorDouble,
    Amf3VectorObject,
    Amf3Dictionary,
}

impl fmt::Display for ObjectType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ObjectType::Amf0Number(_) => write!(f, "Amf0 Number"),
            ObjectType::Amf0Bool(_) => write!(f, "Amf0 Bool"),
            ObjectType::Amf0String(_) => write!(f, "Amf0 String"),
            ObjectType::Amf0Object => write!(f, "Amf0 Object"),
            ObjectType::Amf0Null => write!(f, "Amf0 Null"),
            ObjectType::Amf0Undefined => write!(f, "Amf0 Undefined"),
            ObjectType::Amf0EcmaArray => write!(f, "Amf0 ECMA Array"),
            ObjectType::Amf0StrictArray => write!(f, "Amf0 Strict Array"),
            ObjectType::Amf0Date => write!(f, "Amf0 Date"),
            ObjectType::Amf0LongString => write!(f, "Amf0 Long String"),
            ObjectType::Amf0XML => write!(f, "Amf0 XML"),
            ObjectType::Amf0TypedObject => write!(f, "Amf0 Typed Object"),
            ObjectType::Amf0Switch => write!(f, "Switch to AMF3"),
            ObjectType::Amf3Undefined => write!(f, "Amf3 Undefined"),
            ObjectType::Amf3Null => write!(f, "Amf3 Null"),
            ObjectType::Amf3False => write!(f, "Amf3 False"),
            ObjectType::Amf3True => write!(f, "Amf3 True"),
            ObjectType::Amf3Integer(_) => write!(f, "Amf3 Integer"),
            ObjectType::Amf3Double(_) => write!(f, "Amf3 Double"),
            ObjectType::Amf3String(_) => write!(f, "Amf3 String"),
            ObjectType::Amf3XMLDocument => write!(f, "Amf3 XML Document"),
            ObjectType::Amf3Date => write!(f, "Amf3 Date"),
            ObjectType::Amf3Array(_) => write!(f, "Amf3 Array"),
            ObjectType::Amf3Object(_) => write!(f, "Amf3 Object"),
            ObjectType::Amf3XML => write!(f, "Amf3 XML"),
            ObjectType::Amf3ByteArray => write!(f, "Amf3 Byte Array"),
            ObjectType::Amf3VectorInt => write!(f, "Amf3 Vector Int"),
            ObjectType::Amf3VectorUInt => write!(f, "Amf Vector Unsigned Int"),
            ObjectType::Amf3VectorDouble => write!(f, "Amf3 Vector Double"),
            ObjectType::Amf3VectorObject => write!(f, "Amf3 Vector Object"),
            ObjectType::Amf3Dictionary => write!(f, "Amf3 Dictionary"),
        }
    }
}
