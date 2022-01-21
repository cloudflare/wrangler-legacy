use std::fmt;

use serde::{Deserialize, Serialize};

use crate::settings::binding::Binding;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ConfigR2Bucket {
    pub binding: String,
    pub bucket_name: Option<String>,
    pub preview_bucket_name: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct R2Bucket {
    pub binding: String,
    pub bucket_name: String,
}

impl fmt::Display for R2Bucket {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "binding: {}, bucket_name: {}",
            self.binding, self.bucket_name
        )
    }
}

impl R2Bucket {
    pub fn binding(&self) -> Binding {
        Binding::new_r2_bucket(self.binding.clone(), self.bucket_name.clone())
    }
}
