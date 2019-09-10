#![allow(dead_code)]
use super::emoji;

fn message(msg: &str) {
    println!("{}", msg);
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
