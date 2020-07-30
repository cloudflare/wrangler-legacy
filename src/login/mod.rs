//use crate::terminal::{interactive::confirm, open_browser};

//use openssl::base64;
use openssl::rsa::Rsa;

pub fn run() -> Result<(), failure::Error> {
    let rsa = Rsa::generate(1024)?;
    let _ = rsa.public_key_to_pem_pkcs1()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use openssl::rsa::Rsa;

    #[test]
    fn test_rsa() {
        let rsa = Rsa::generate(1024).unwrap();
        rsa.public_key_to_pem_pkcs1().unwrap();
    }
}
