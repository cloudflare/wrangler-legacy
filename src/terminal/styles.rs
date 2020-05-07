use console::{style, StyledObject};

pub fn url(msg: &str) -> StyledObject<&str> {
    style(msg).blue().bold()
}

pub fn warning(msg: &str) -> StyledObject<&str> {
    style(msg).red().bold()
}

pub fn highlight(msg: &str) -> StyledObject<&str> {
    style(msg).yellow().bold()
}
