use crate::commands;
use crate::settings::toml::TargetType;

use anyhow::Result;

pub fn generate(
    name: String,
    site: bool,
    template: Option<String>,
    target_type: Option<TargetType>,
) -> Result<()> {
    const DEFAULT_TEMPLATE: &str = "https://github.com/cloudflare/worker-template";
    const RUST_TEMPLATE: &str = "https://github.com/cloudflare/rustwasm-worker-template";
    const SITES_TEMPLATE: &str = "https://github.com/cloudflare/worker-sites-template";

    let template = if site {
        SITES_TEMPLATE
    } else if let Some(template) = template.as_deref() {
        template
    } else if let Some(TargetType::Rust) = target_type {
        RUST_TEMPLATE
    } else {
        DEFAULT_TEMPLATE
    };

    log::info!(
        "Generate command called with template {}, and name {}",
        template,
        name
    );

    commands::generate(&name, template, target_type, site)
}
