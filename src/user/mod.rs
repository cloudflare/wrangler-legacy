use account::Account;
use settings::Settings;

mod account;
pub mod settings;

pub struct User {
    pub account: Account,
    pub settings: Settings,
}

impl User {
    pub fn new() -> Result<User, failure::Error> {
        let settings = Settings::new()
            .expect("🚧 Whoops! You aren't configured yet. Run `wrangler config`! 🚧");

        let account = Account::new(settings.clone())?;
        Ok(User { account, settings })
    }
}
