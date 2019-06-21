use super::emoji;
use console::Emoji;

fn message(label: Emoji, msg: &str) {
    println!("{}", format!("{0} {1} {0}", label, msg));
}

pub fn info(msg: &str) {
    message(emoji::INFO, msg);
}

pub fn success(msg: &str) {
    message(emoji::SPARKLES, msg);
}

pub fn warn(msg: &str) {
    message(emoji::WARN, msg);
}

pub fn user_error(msg: &str) {
    message(emoji::EYES, msg);
}

pub fn service_error(msg: &str) {
    message(emoji::FACEPALM, msg);
}

pub fn working(msg: &str) {
    message(emoji::SWIRL, msg);
}

pub fn preview(msg: &str) {
    message(emoji::WORKER, msg);
}
