use regex::Regex;

pub fn preprocess_markdown(text: &str) -> String {

     // Remove spaces between quotes and punctuation
    let contents = remove_spaces_between_quotes_and_punctuation(&text);

    // Replace quotes and apostrophes with curly ones and fix punctuation placement
    let replaced = replace_quotes_and_fix_punctuation(&contents);

    // Remove extra spaces
    let no_extra_spaces = remove_extra_spaces(&replaced);

    let result = replace_breaks(&no_extra_spaces);

    result
}

fn replace_breaks(text: &str) -> String {
    text.replace("\n----\n", "\n\n&nbsp;\n\n<div class=\"center\">***</div>\n\n&nbsp;\n\n")
}

fn remove_spaces_between_quotes_and_punctuation(text: &str) -> String {
    let re = Regex::new(r#""\s+([.,!?…])"#).unwrap();
    re.replace_all(text, "\"$1").into_owned()
}

fn replace_quotes_and_fix_punctuation(text: &str) -> String {
    let mut output = String::new();
    let mut inside_quote = false;
    let mut inside_html_tag = false;

    let mut chars: Vec<char> = text.chars().collect();
    let length = chars.len();

    for i in 0..length {
        let ch = chars[i];

        if ch == '<' {
            inside_html_tag = true;
        } else if ch == '>' {
            inside_html_tag = false;
        }

        if inside_html_tag {
            output.push(ch);
            continue;
        }

        match ch {
            '\'' => {
                if i > 0 && chars[i - 1].is_alphabetic() &&
                    i < length - 1 && chars[i + 1].is_alphabetic() {
                    output.push('’');
                } else if inside_quote && i < length - 1 && is_punctuation(chars[i + 1]) {
                    // If the next character is punctuation, move it before the quote
                    output.push(chars[i + 1]);
                    output.push('’');
                    chars[i + 1] = ' '; // replace punctuation with space
                    continue;
                } else {
                    output.push('‘');
                }
            }
            '"' => {
                inside_quote = !inside_quote;

                if i < length - 1 && is_punctuation(chars[i + 1]) {
                    // If the next character is punctuation, move it before the quote
                    output.push(chars[i + 1]);
                    output.push('”');
                    chars[i + 1] = ' '; // replace punctuation with space
                    continue;
                } else {
                    output.push('“');
                }
            }
            _ => output.push(ch),
        }
    }

    output
}

fn xxxreplace_quotes_and_fix_punctuation(text: &str) -> String {
    let mut output = String::new();
    let mut inside_quote = false;

    let mut chars: Vec<char> = text.chars().collect();
    let length = chars.len();

    for i in 0..length {
        let ch = chars[i];
        match ch {
            '\'' => {
                if i > 0 && chars[i - 1].is_alphabetic() &&
                    i < length - 1 && chars[i + 1].is_alphabetic() {
                    output.push('’');
                } else if inside_quote && i < length - 1 && is_punctuation(chars[i + 1]) {
                    // If the next character is punctuation, move it before the quote
                    output.push(chars[i + 1]);
                    output.push('’');
                    chars[i + 1] = ' '; // replace punctuation with space
                    continue;
                } else {
                    output.push('‘');
                }
            }
            '"' => {
                inside_quote = !inside_quote;

                if i < length - 1 && is_punctuation(chars[i + 1]) {
                    // If the next character is punctuation, move it before the quote
                    output.push(chars[i + 1]);
                    output.push('”');
                    chars[i + 1] = ' '; // replace punctuation with space
                    continue;
                } else {
                    output.push('“');
                }
            }
            _ => output.push(ch),
        }
    }

    output
}

fn remove_extra_spaces(text: &str) -> String {
    text.lines()
        .map(|line| line.split_whitespace().collect::<Vec<&str>>().join(" "))
        .collect::<Vec<String>>()
        .join("\n")
}

fn is_punctuation(ch: char) -> bool {
    match ch {
        ',' | '.' | '?' | '!' | ';' | ':' | '…' => true,
        _ => false,
    }
}
