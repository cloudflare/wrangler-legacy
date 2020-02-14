use crate::terminal::emoji;

// make this an even number
const OUTER_PADDING: usize = 15;

const UNSTABLE_MSG: &str =
    "wrangler dev is currently unstable and there are likely to be breaking changes!";
const INTEGRATION_MSG: &str =
    "For this reason, we cannot yet recommend using wrangler dev for integration testing";
const FEEDBACK_MSG: &str =
    "Please submit any feedback here: https://github.com/cloudflare/wrangler/issues/1047";

pub fn dev_alpha_warning() {
    if let Some((width, _)) = term_size::dimensions() {
        if width >= 70 {
            print_top(width);
            print_centered(width, "");
            print_centered(width, UNSTABLE_MSG);
            print_centered(width, INTEGRATION_MSG);
            print_centered(width, FEEDBACK_MSG);
            print_centered(width, "");
            print_bottom(width);
            println!("");
        } else {
            print_unboxed();
        }
    } else {
        print_unboxed();
    }
}

fn print_unboxed() {
    println!(
        "{0} {1}\n{0} {2}\n\n{3} {4}",
        emoji::WARN,
        UNSTABLE_MSG,
        INTEGRATION_MSG,
        emoji::INFO,
        FEEDBACK_MSG
    );
}

fn print_centered(width: usize, text: &str) {
    let text = text.to_string();
    let text_length = text.len();
    let available_space = width - OUTER_PADDING;
    let mut words = vec!["".to_string()];
    let mut counter = 0;
    if text_length < available_space {
        words[0] = text;
    } else {
        for word in text.split_whitespace() {
            let candidate = format!("{} {}", words[counter], word);
            let candidate_length = candidate.len();
            if candidate_length < available_space && candidate_length < (text_length / 2) + 2 {
                words[counter] = candidate;
            } else {
                counter += 1;
                words.push(word.to_string());
            }
        }
        words.push("".to_string());
    }
    for word in words {
        print_line(available_space, &word);
    }
}

fn print_line(space: usize, text: &str) {
    let total_spaces = space - text.len();
    let padding = (space - text.len()) / 2;
    print_outer_padding();
    print!("│");
    for _ in 0..padding {
        print!(" ");
    }
    print!("{}", text);
    for _ in 0..padding {
        print!(" ");
    }
    if padding * 2 != total_spaces {
        print!(" ");
    }
    print!("│\n");
}

fn print_outer_padding() {
    for _ in 0..(OUTER_PADDING / 2) - 1 {
        print!(" ")
    }
}

fn print_top(width: usize) {
    print_outer_padding();
    print!("┌");
    print_dash(width - OUTER_PADDING);
    print!("┐\n");
}

fn print_bottom(width: usize) {
    print_outer_padding();
    print!("└");
    print_dash(width - OUTER_PADDING);
    print!("┘\n");
}

fn print_dash(num: usize) {
    for _ in 0..num {
        print!("─");
    }
}
