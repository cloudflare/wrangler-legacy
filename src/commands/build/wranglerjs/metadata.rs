use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct Bindings {
    pub name: String,
    #[serde(rename = "type")]
    pub binding_type: String,
    pub part: Option<String>,
}

#[derive(Serialize, Debug)]
pub struct Metadata {
    pub body_part: String,
    pub bindings: Vec<Bindings>,
}
