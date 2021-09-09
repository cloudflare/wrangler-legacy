use colored_json::{Color, ColoredFormatter, PrettyFormatter, Styler};

pub fn colored_json_string(value: &serde_json::Value) -> Result<String, serde_json::Error> {
    let formatter = ColoredFormatter::with_styler(
        PrettyFormatter::new(),
        Styler {
            key: Color::Green.normal(),
            string_value: Color::Blue.bold(),
            integer_value: Color::Purple.bold(),
            float_value: Color::Purple.italic(),
            object_brackets: Color::Yellow.bold(),
            array_brackets: Color::Cyan.bold(),
            bool_value: Color::Red.bold(),
            ..Default::default()
        },
    );

    formatter.to_colored_json_auto(value)
}
