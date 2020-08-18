use super::emoji;
use once_cell::sync::OnceCell;

use billboard::{Billboard, BorderColor, BorderStyle};
use serde::{Deserialize, Serialize};

#[derive(PartialEq)]
pub enum OutputType {
    Json,
    Human,
}

#[derive(Serialize, Deserialize)]
pub struct PublishOutput {
    pub success: Option<bool>,
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub urls: Vec<String>,
}

static OUTPUT_TYPE: OnceCell<OutputType> = OnceCell::new();

pub fn set_output_type(typ: OutputType) {
    match OUTPUT_TYPE.set(typ) {
        Ok(_) => {}
        Err(_) => {
            message(&"Output type already set".to_string());
        }
    }
}

pub fn is_json_output() -> bool {
    OUTPUT_TYPE.get() == Some(&OutputType::Json)
}

// Always goes to stdout
pub fn jsonout<T>(value: &T)
where
    T: ?Sized + Serialize,
{
    println!("{}", &serde_json::to_string(value).unwrap());
}

fn message(msg: &str) {
    match OUTPUT_TYPE.get() {
        Some(OutputType::Json) => {
            eprintln!("{}", msg);
        }
        _ => {
            println!("{}", msg);
        }
    }
}

pub fn billboard(msg: &str) {
    let billboard = Billboard::builder()
        .border_style(BorderStyle::Round)
        .border_color(BorderColor::Cyan)
        .margin(1)
        .build();
    billboard.display(msg);
}

pub fn info(msg: &str) {
    let msg = format!("{} {}", emoji::INFO, msg);
    message(&msg);
}

pub fn warn(msg: &str) {
    let msg = format!("{} {}", emoji::WARN, msg);
    message(&msg);
}

pub fn success(msg: &str) {
    let msg = format!("{} {}", emoji::SPARKLES, msg);
    message(&msg);
}

pub fn user_error(msg: &str) {
    let msg = format!("{} {}", emoji::EYES, msg);
    message(&msg);
}

pub fn working(msg: &str) {
    let msg = format!("{} {}", emoji::SWIRL, msg);
    message(&msg);
}

pub fn preview(msg: &str) {
    let msg = format!("{} {}", emoji::WORKER, msg);
    message(&msg);
}

pub fn help(msg: &str) {
    let msg = format!("{} {}", emoji::SLEUTH, msg);
    message(&msg);
}

pub fn deprecation_warning(msg: &str) {
    let bb = Billboard::builder()
        .border_style(BorderStyle::Round)
        .border_color(BorderColor::Red)
        .margin(1)
        .build();
    bb.display(msg);
}
