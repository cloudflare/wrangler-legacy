use super::file::{File, ToFile};

#[derive(Debug)]
pub struct Script {
    pub path: String,
}

impl ToFile for Script {
    fn to_file(&self) -> File {
        File {
            name: "script".to_string(),
            path: self.path.clone(),
        }
    }
}
