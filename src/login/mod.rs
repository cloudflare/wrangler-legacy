use eventual::Timer;
use indicatif::{ProgressBar, ProgressStyle};
use openssl::base64;
use openssl::rsa::{Padding, Rsa};
use percent_encoding::{percent_encode, NON_ALPHANUMERIC};
use std::collections::HashMap;
use std::str;

use crate::commands::config::global_config;
use crate::settings::global_user::GlobalUser;
use crate::terminal::{interactive, message, open_browser};

pub fn run() -> Result<(), failure::Error> {
    let rsa = Rsa::generate(1024)?;
    let pubkey = rsa.public_key_to_pem_pkcs1()?;

    // Convert key to string and remove header and footer
    let pubkey_str = str::from_utf8(&pubkey)?;
    let pubkey_filtered = pubkey_str
        .lines()
        .filter(|line| !line.starts_with("-"))
        .fold(String::new(), |mut data, line| {
            data.push_str(&line);
            data
        });
    let pubkey_encoded = percent_encode(pubkey_filtered.as_bytes(), NON_ALPHANUMERIC).to_string();

    let browser_permission =
        interactive::confirm("Allow Wrangler to a open page in your browser?")?;
    if !browser_permission {
        failure::bail!("In order to use login you must allow Wrangler to open pages in your browser. If you don't want to do this consder using `wrangler config`");
    }

    open_browser(&format!(
        "https://dash.staging.cloudflare.com/wrangler?key={0}",
        pubkey_encoded
    ))?;

    let encrypted_token_str = poll_token(pubkey_filtered)?;
    let encrypted_token = base64::decode_block(encrypted_token_str.as_str())?;
    let mut token_bytes: [u8; 128] = [0; 128];

    rsa.private_decrypt(&encrypted_token, &mut token_bytes, Padding::PKCS1)?;
    let token = str::from_utf8(&token_bytes)?;

    let user = GlobalUser::TokenAuth {
        api_token: token.to_string(),
    };
    global_config(&user, true)?;

    Ok(())
}

/// Poll for token, bail after 500 seconds.
fn poll_token(token_id: String) -> Result<String, failure::Error> {
    let mut request_params = HashMap::new();
    request_params.insert("token-id", token_id);

    let client = reqwest::blocking::Client::new();
    let timer = Timer::new().interval_ms(1000).iter();

    let style = ProgressStyle::default_spinner().template("{spinner}   {msg}");
    let spinner = ProgressBar::new_spinner().with_style(style);
    spinner.set_message("Waiting to be sent api token...");
    spinner.enable_steady_tick(20);

    let mut seconds = 0;

    for _ in timer {
        let res = client
            .get("https://api.staging.cloudflare.com/client/v4/workers/token")
            .json(&request_params)
            .send()?;

        if res.status().is_success() {
            return Ok(res.text_with_charset("utf-8")?);
        }

        if seconds >= 500 {
            break;
        }
        seconds += 1;
    }

    failure::bail!(
        "Timed out when waiting for api token. Try using `wrangler config` if login fails to work."
    );
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
