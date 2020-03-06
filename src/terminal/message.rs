use super::emoji;

use boxx::{BorderColor, BorderStyle, Boxx};

fn message(msg: &str) {
    println!("{}", msg);
}

pub fn billboard(msg: &str) {
    let billboard = Boxx::builder()
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
    let msg = format!("\n\t{} {}", emoji::WARN, msg);
    message(&msg);
}
