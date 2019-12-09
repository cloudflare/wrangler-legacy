mod details;

use serde::{Deserialize, Serialize};
use std::fmt;

use details::ExceptionDetails;

#[derive(Debug, Serialize, Deserialize)]
pub struct ExceptionEvent {
    #[serde(rename = "exceptionDetails")]
    pub details: ExceptionDetails,
}

impl fmt::Display for ExceptionEvent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", &self.details.text)?;
        if let Some(description) = &self.details.exception.description {
            write!(f, "\n{}", description)?;
        };
        Ok(())
    }
}
