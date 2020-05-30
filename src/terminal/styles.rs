use console::{style, StyledObject};

pub fn url<D>(msg: D) -> StyledObject<D> {
    style(msg).blue().bold()
}

pub fn warning<D>(msg: D) -> StyledObject<D> {
    style(msg).red().bold()
}

pub fn highlight<D>(msg: D) -> StyledObject<D> {
    style(msg).yellow().bold()
}
