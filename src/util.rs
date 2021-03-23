use clap::App;
pub trait ApplyToApp<'a, 'b> {
    fn apply(self, f: fn(App<'a, 'b>) -> App<'a, 'b>) -> App<'a, 'b>;
}

impl<'a, 'b> ApplyToApp<'a, 'b> for App<'a, 'b> {
    fn apply(self, f: fn(App<'a, 'b>) -> App<'a, 'b>) -> App<'a, 'b> {
        f(self)
    }
}
