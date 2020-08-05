use openssl::asn1::Asn1Time;
use openssl::bn::{BigNum, MsbOption};
use openssl::hash::MessageDigest;
use openssl::pkey::PKey;
use openssl::rsa::Rsa;
use openssl::x509::extension::{BasicConstraints, KeyUsage, SubjectKeyIdentifier};
use openssl::x509::{X509NameBuilder, X509};
use std::fs;
use std::path::PathBuf;

use crate::settings::get_wrangler_home_dir;
use crate::terminal::message;

/// Create files for cert and private key
fn create_output_files() -> Result<Option<(PathBuf, PathBuf)>, failure::Error> {
    let home = get_wrangler_home_dir()?.join("config");
    let cert = home.join("dev-cert.pem");
    let privkey = home.join("dev-privkey.rsa");

    if cert.exists() && privkey.exists() {
        Ok(None)
    } else {
        fs::create_dir_all(&home)?;

        message::info(format!("Generating certificate and private key for https server, if you would like to use your own you can replace `dev-cert.pem` and `dev-privkey.rsa` at {}", home.to_str().unwrap()).as_str());

        Ok(Some((cert, privkey)))
    }
}

/// Generate cert and private key for local https server
pub fn generate_cert() -> Result<(), failure::Error> {
    let files = create_output_files()?;
    if files.is_none() {
        return Ok(());
    }

    let (cert_file, priv_file) = files.unwrap();

    let rsa = Rsa::generate(2048)?;
    let privkey = PKey::from_rsa(rsa)?;

    let mut x509_name = X509NameBuilder::new()?;
    x509_name.append_entry_by_text("C", "US")?;
    x509_name.append_entry_by_text("ST", "TX")?;
    x509_name.append_entry_by_text("O", "Example")?;
    x509_name.append_entry_by_text("CN", "example")?;
    let x509_name = x509_name.build();

    let mut cert_builder = X509::builder()?;
    cert_builder.set_version(2)?;
    let serial_number = {
        let mut serial = BigNum::new()?;
        serial.rand(159, MsbOption::MAYBE_ZERO, false)?;
        serial.to_asn1_integer()?
    };
    cert_builder.set_serial_number(&serial_number)?;
    cert_builder.set_subject_name(&x509_name)?;
    cert_builder.set_issuer_name(&x509_name)?;
    cert_builder.set_pubkey(&privkey)?;
    let not_before = Asn1Time::days_from_now(0)?;
    cert_builder.set_not_before(&not_before)?;
    let not_after = Asn1Time::days_from_now(365)?;
    cert_builder.set_not_after(&not_after)?;

    cert_builder.append_extension(BasicConstraints::new().critical().ca().build()?)?;
    cert_builder.append_extension(
        KeyUsage::new()
            .critical()
            .key_cert_sign()
            .crl_sign()
            .build()?,
    )?;

    let subject_key_identifier =
        SubjectKeyIdentifier::new().build(&cert_builder.x509v3_context(None, None))?;
    cert_builder.append_extension(subject_key_identifier)?;

    cert_builder.sign(&privkey, MessageDigest::sha256())?;
    let cert_str = cert_builder.build().to_pem().unwrap();

    let priv_str = privkey.private_key_to_pem_pkcs8().unwrap();

    fs::write(cert_file, cert_str)?;
    fs::write(priv_file, priv_str)?;

    Ok(())
}
