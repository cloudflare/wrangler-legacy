mod console;
mod exception;

use self::console::ConsoleEvent;
use self::exception::ExceptionEvent;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "method", content = "params")]
pub enum DevtoolsEvent {
    #[serde(rename = "Runtime.consoleAPICalled")]
    ConsoleAPICalled(ConsoleEvent),
    #[serde(rename = "Runtime.exceptionThrown")]
    ExceptionThrown(ExceptionEvent),
}
