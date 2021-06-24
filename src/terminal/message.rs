use super::emoji;

use billboard::{Billboard, BorderColor, BorderStyle};
use serde::Serialize;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Output {
    Json,
    PlainText,
}
pub trait Message {
    fn message(msg: &str);

    fn info(msg: &str) {
        let msg = format!("{} {}", emoji::INFO, msg);
        Self::message(&msg);
    }

    fn warn(msg: &str) {
        let msg = format!("{} {}", emoji::WARN, msg);
        Self::message(&msg);
    }

    fn success(msg: &str) {
        let msg = format!("{} {}", emoji::SPARKLES, msg);
        Self::message(&msg);
    }

    fn user_error(msg: &str) {
        let msg = format!("{} {}", emoji::EYES, msg);
        Self::message(&msg);
    }

    fn working(msg: &str) {
        let msg = format!("{} {}", emoji::SWIRL, msg);
        Self::message(&msg);
    }

    fn preview(msg: &str) {
        let msg = format!("{} {}", emoji::WORKER, msg);
        Self::message(&msg);
    }

    fn help(msg: &str) {
        let msg = format!("{} {}", emoji::SLEUTH, msg);
        Self::message(&msg);
    }

    fn billboard(msg: &str);
    fn deprecation_warning(msg: &str);
    #[allow(clippy::wrong_self_convention)]
    fn as_json<T>(value: &T)
    where
        T: ?Sized + Serialize;
}

pub struct StdOut;

impl Message for StdOut {
    fn message(msg: &str) {
        println!("{}", msg);
    }

    fn billboard(msg: &str) {
        let billboard = Billboard::builder()
            .border_style(BorderStyle::Round)
            .border_color(BorderColor::Cyan)
            .margin(1)
            .build();
        billboard.display(msg);
    }

    fn deprecation_warning(msg: &str) {
        let bb = Billboard::builder()
            .border_style(BorderStyle::Round)
            .border_color(BorderColor::Red)
            .margin(1)
            .build();
        bb.display(msg);
    }

    fn as_json<T>(value: &T)
    where
        T: ?Sized + Serialize,
    {
        println!("{}", &serde_json::to_string(value).unwrap());
    }
}

pub struct StdErr;

impl Message for StdErr {
    fn message(msg: &str) {
        eprintln!("{}", msg);
    }

    fn billboard(_msg: &str) {
        panic!("Can't display billboard to stderr.")
    }

    fn deprecation_warning(_msg: &str) {
        panic!("Can't display billboard warning to stderr.")
    }
    fn as_json<T>(_value: &T)
    where
        T: ?Sized + Serialize,
    {
        panic!("Json output intended for stdout, not stderr.")
    }
}
