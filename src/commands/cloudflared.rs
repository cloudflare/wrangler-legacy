use crate::install;
use crate::commands;

pub fn run_cloudflared() -> Result<(), failure::Error> {
    let tool_name = "cloudflared";
    let binary_path = install::install(tool_name, "cloudflare")?.binary(tool_name)?;

    let args = [];
    let command = commands::build::command(&args, &binary_path);
    let command_name = format!("{:?}", command);

    commands::run(command, &command_name)?;
    Ok(())
}