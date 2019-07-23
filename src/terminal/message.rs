use super::emoji;
use console::Emoji;

fn message(label: Emoji, msg: &str) {
    println!("{0} {1} {0}", label, msg);
}

pub fn info(msg: &str) {
    message(emoji::INFO, msg);
}

pub fn success(msg: &str) {
    message(emoji::SPARKLES, msg);
}

pub fn user_error(msg: &str) {
    message(emoji::EYES, msg);
}

pub fn working(msg: &str) {
    message(emoji::SWIRL, msg);
}

pub fn preview(msg: &str) {
    message(emoji::WORKER, msg);
}

pub fn warn(msg: &str) {
    message(emoji::WARN, msg);
}
