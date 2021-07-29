use crate::{commands, settings::toml::TargetType};
use anyhow::Result;

pub fn init(name: Option<String>, site: bool, target_type: Option<TargetType>) -> Result<()> {
    let target_type = if site {
        // Workers Sites projects are always webpack for now
        Some(TargetType::Webpack)
    } else {
        target_type
    };

    commands::init(name.as_deref(), target_type, site)
}
