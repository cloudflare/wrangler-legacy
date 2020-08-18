use super::emoji;
use once_cell::sync::OnceCell;

use billboard::{Billboard, BorderColor, BorderStyle};

pub enum OutputType {
    Json,
    Human
}

static OUTPUT_TYPE: OnceCell<OutputType> = OnceCell::new();

pub fn set_output_type(typ: OutputType) {
    OUTPUT_TYPE.set(typ);
}

fn message(msg: &str) {
    match OUTPUT_TYPE.get() {
        Some(OutputType::Json) => {
            println!("json");
            eprintln!("{}", msg);
        }
        Some(OutputType::Human) => {
            println!("human");
            println!("{}", msg);
        }
        _ => {
            panic!("output not defined")
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
