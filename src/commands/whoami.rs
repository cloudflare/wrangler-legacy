use crate::settings::Settings;
use reqwest::header::CONTENT_TYPE;

pub fn whoami(settings: Settings) -> Result<(), failure::Error> {
    let user_addr = "https://api.cloudflare.com/client/v4/user";

    let client = reqwest::Client::new();

    let res = client
        .get(user_addr)
        .header("X-Auth-Key", settings.api_key)
        .header("X-Auth-Email", settings.email)
        .header(CONTENT_TYPE, "application/json")
        .send();

    println!("{:?}", &res?.text());
    Ok(())
}
