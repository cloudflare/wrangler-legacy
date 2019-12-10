mod array;
mod description;
mod map;

use serde::{Deserialize, Serialize};
use std::fmt;

use array::ArrayData;
use description::Description;
use map::MapData;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "subtype", rename_all = "lowercase")]
pub enum Subtype {
    Array(ArrayData),
    Map(MapData),
    RegExp(Description),
    Date(Description),
    Set(Description),
    WeakMap(Description),
    WeakSet(Description),
    #[serde(rename = "iterator")]
    JsIterator(Description),
    Generator(Description),
    Error(Description),
    Proxy(Description),
    Promise(Description),
    TypedArray(Description),
    ArrayBuffer(Description),
    DataView(Description),
    Null,
}

impl fmt::Display for Subtype {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            Subtype::Array(array) => write!(f, "{}", array),
            Subtype::Null => write!(f, "null"),
            Subtype::RegExp(reg_exp) => write!(f, "{}", reg_exp),
            Subtype::Date(date) => write!(f, "{}", date),
            Subtype::Map(map) => write!(f, "{}", map),
            _ => write!(f, "unhandled type"),
        }
    }
}
