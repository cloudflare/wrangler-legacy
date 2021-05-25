use clap::App;
pub trait ApplyToApp<'a, 'b> {
    fn apply(self, f: fn(App<'a, 'b>) -> App<'a, 'b>) -> App<'a, 'b>;
}

impl<'a, 'b> ApplyToApp<'a, 'b> for App<'a, 'b> {
    fn apply(self, f: fn(App<'a, 'b>) -> App<'a, 'b>) -> App<'a, 'b> {
        f(self)
    }
}

pub const TEMP_NOTICE_ES_MODULES_DO_BETA: &str = "Your account does not have permission to do this! While Durable Objects are in Beta, the modules format is limited to accounts which have opted-in to the Beta. You may do so by following the instructions here: https://developers.cloudflare.com/workers/learning/using-durable-objects";
