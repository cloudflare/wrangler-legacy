mod properties;

use serde::{Deserialize, Serialize};

use properties::ObjectProperties;

#[derive(Debug, Serialize, Deserialize)]
pub struct Object {
    pub preview: ObjectPreview,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ObjectPreview {
    #[serde(rename = "type")]
    pub object_type: String,
    pub description: String,
    pub overflow: bool,
    pub properties: Vec<ObjectProperties>,
}
