use anyhow::Result;
use atty::Stream;
use std::io::{self, Read};

// For interactively handling reading in a string
pub fn get_user_input(prompt_string: &str) -> String {
    println!("{}", prompt_string);
    let mut input: String = read!("{}\n");
    input = strip_trailing_whitespace(input);
    input
}

pub fn get_user_input_multi_line(prompt_string: &str) -> String {
    println!("{}", prompt_string);
    // are we reading from user input?
    let mut input = if atty::is(Stream::Stdin) {
        read!("{}\n")
    } else {
        // or is this data from a pipe? (support newlines)
        let mut tmp = String::new();
        let _ = io::stdin().read_to_string(&mut tmp);
        tmp
    };
    input = strip_trailing_whitespace(input);
    input
}

fn strip_trailing_whitespace(mut input: String) -> String {
    input.truncate(input.trim_end().len());
    input
}

// Truncate all "yes", "no" responses for interactive prompt to just "y" or "n".
const INTERACTIVE_RESPONSE_LEN: usize = 1;
const YES: &str = "y";
const NO: &str = "n";
// For interactively handling destructive commands (and discouraging accidental deletes).
// Input like "yes", "Yes", "no", "No" will be accepted, thanks to the whitespace-stripping
// and lowercasing logic below.
// TODO: loop this to retry until valid input is received.
pub fn confirm(prompt_string: &str) -> Result<bool> {
    println!("{} [y/n]", prompt_string);
    let mut response: String = read!("{}\n");
    response = response.split_whitespace().collect(); // remove whitespace
    response.make_ascii_lowercase(); // ensure response is all lowercase
    response.truncate(INTERACTIVE_RESPONSE_LEN); // at this point, all valid input will be "y" or "n"
    match response.as_ref() {
        YES => Ok(true),
        NO => Ok(false),
        _ => anyhow::bail!("Response must either be \"y\" for yes or \"n\" for no"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_trims_user_input_right_whitespace_chars() {
        let test_str = "mysecret\r".to_string();

        let truncated_str = strip_trailing_whitespace(test_str);
        assert_eq!(truncated_str, "mysecret")
    }
}
