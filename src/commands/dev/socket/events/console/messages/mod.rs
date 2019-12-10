mod bigint;
mod boolean;
mod function;
mod number;
mod objects;
mod string;
mod symbol;

use bigint::BigIntData;
use boolean::BooleanData;
use function::FunctionData;
use number::NumberData;
use objects::ObjectData;
use string::StringData;
use symbol::SymbolData;

use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum LogMessage {
    Object(ObjectData),
    Number(NumberData),
    BigInt(BigIntData),
    Boolean(BooleanData),
    #[serde(rename = "string")]
    StringJs(StringData),
    Symbol(SymbolData),
    Undefined,
    Function(FunctionData),
}

impl fmt::Display for LogMessage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            LogMessage::Object(object) => write!(f, "{}", object),
            LogMessage::Boolean(boolean) => write!(f, "{}", boolean),
            LogMessage::StringJs(string) => write!(f, "{}", string),
            LogMessage::Undefined => write!(f, "undefined"),
            LogMessage::Function(function) => write!(f, "{}", function),
            LogMessage::Number(number) => write!(f, "{}", number),
            LogMessage::Symbol(symbol) => write!(f, "{}", symbol),
            LogMessage::BigInt(bigint) => write!(f, "{}", bigint),
        }
    }
}
