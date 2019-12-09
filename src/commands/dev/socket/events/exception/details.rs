use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ExceptionDetails {
    pub text: String,
    pub exception: Exception,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Exception {
    pub description: Option<String>,
}
