use anyhow::Result;
use url::Url;

pub trait Session: Sized {
    fn get_socket_url(&self) -> &Url;
    fn refresh(&self) -> Result<Self>;
}
