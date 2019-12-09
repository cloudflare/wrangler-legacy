mod array;

use serde::{Deserialize, Serialize};
use std::fmt;

use array::ArrayData;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "subtype", rename_all = "lowercase")]
pub enum Subtype {
    Array(ArrayData),
    Null,
    Node,
    RegExp,
    Date,
    Map,
    Set,
    WeakMap,
    WeakSet,
    #[serde(rename = "iterator")]
    JsIterator,
    Generator,
    Error,
    Proxy,
    Promise,
    TypedArray,
    ArrayBuffer,
    DataView,
}

impl fmt::Display for Subtype {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            Subtype::Array(array) => write!(f, "{}", array),
            Subtype::Null => write!(f, "null"),
            _ => write!(f, "{:?}", &self),
        }
    }
}
