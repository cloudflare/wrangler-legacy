//! Localization for `wrangler`

use i18n_embed::{
    fluent::{fluent_language_loader, FluentLanguageLoader},
    LanguageLoader,
};
use lazy_static::lazy_static;
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "i18n/"]
pub struct Localizations;

lazy_static! {
    static ref LOADER: FluentLanguageLoader = initalize_i18n();
}

fn initalize_i18n() -> FluentLanguageLoader {
    let loader: FluentLanguageLoader = fluent_language_loader!();
    loader
        .load_languages(&Localizations, &[loader.fallback_language()])
        .expect("Could not embed i18n localizations");
    loader
}

#[macro_export]
macro_rules! i18n {
    ($message_id:literal) => {{
        i18n_embed_fl::fl!($crate::i18n::LOADER, $message_id)
    }};

    ($message_id:literal, $($args:expr),*) => {{
        i18n_embed_fl::fl!($crate::i18n::LOADER, $message_id, $($args), *)
    }};
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_localizes_a_simple_definition() {
        assert_eq!(
            i18n!("hello-user", name = "Ferris"),
            "Hello \u{2068}Ferris\u{2069}, welcome to wrangler!"
        );
    }

    #[test]
    fn it_localizes_a_complex_definition() {
        assert_eq!(
            i18n!("hello-script", count = 0),
            "You have \u{2068}no scripts.\u{2069}"
        );
        assert_eq!(
            i18n!("hello-script", count = 1),
            "You have \u{2068}1 script.\u{2069}"
        );
        assert_eq!(
            i18n!("hello-script", count = 12),
            "You have \u{2068}\u{2068}12\u{2069} scripts.\u{2069}"
        );
    }
}
