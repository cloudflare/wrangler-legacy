use crate::commands;
use crate::terminal::message::{Message, StdOut};
use crate::terminal::{interactive, styles};

pub fn login(scopes: bool) -> Result<(), anyhow::Error> {
    if scopes {
        let scopes_input: String = interactive::get_user_input("Please enter desired scopes in a whitespace separated list (e.g. \"scope1 scope2 .. scopeN\").");
        let scopes_list: Vec<&str> = scopes_input.split_whitespace().collect();

        if !scopes_list.is_empty() {
            return commands::login::run(Some(scopes_list.as_ref()));
        }

        // User didn't provide any scopes
        StdOut::message(&format!(
            "No scope provided. {} will be configured with the default scopes.",
            styles::highlight("`wrangler login`")
        ));
    }

    // If the user doesn't provide flag or doesn't enter any scopes, fallback to default scopes
    commands::login::run(None)
}
