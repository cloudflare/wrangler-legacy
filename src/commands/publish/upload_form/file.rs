#[derive(Debug)]
pub struct File {
    pub name: String,
    pub path: String,
}

pub trait ToFile {
    fn to_file(&self) -> File;
}
