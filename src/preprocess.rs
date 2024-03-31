use regex::Regex;

pub fn preprocess_markdown(text: &str) -> String {

     // Remove spaces between quotes and punctuation
    let mut contents = remove_spaces_between_quotes_and_punctuation(&text);

    // Replace quotes and apostrophes with curly ones and fix punctuation placement
    contents = replace_quotes(&contents);
    contents = fix_punctuation(&contents);

    // fix anomalies
    contents = fix_em_anomaly(&contents);
    contents = fix_tshirt_anomaly(&contents);

    // Remove extra spaces
    contents = remove_extra_spaces(&contents);

    let result = replace_breaks(&contents);

    result
}

fn replace_breaks(text: &str) -> String {
    text.replace("\n----\n", "\n\n&nbsp;\n\n<div class=\"center\">***</div>\n\n&nbsp;\n\n")
}

// pub fn xxpreprocess_markdown(text: &str) -> String {

//     // Remove spaces between quotes and punctuation
//    let contents = remove_spaces_between_quotes_and_punctuation(&text);

//    // Replace quotes and apostrophes with curly ones and fix punctuation placement
//    let replaced = replace_quotes_and_fix_punctuation(&contents);

//    // Remove extra spaces
//    let no_extra_spaces = remove_extra_spaces(&replaced);

//    let result = replace_breaks(&no_extra_spaces);

//    result
// }

// fn replace_breaks(text: &str) -> String {
//    text.replace("\n----\n", "\n\n&nbsp;\n\n<div class=\"center\">***</div>\n\n&nbsp;\n\n")
// }

fn remove_spaces_between_quotes_and_punctuation(text: &str) -> String {
    let re = Regex::new(r#""\s+([.,!?…])"#).unwrap();
    re.replace_all(text, "\"$1").into_owned()
}

fn replace_quotes(text: &str) -> String {
    let mut output = String::new();
    let mut inside_quote = false;
    let mut inside_html_tag = false;

    for ch in text.chars() {
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
            // Handle single quotes, assuming you want to replace them with curly ones
            '\'' => output.push('’'), // Simplified; might need more logic for apostrophes vs quotes
            // Handle double quotes
            '"' | '“' | '”' => {
                inside_quote = !inside_quote;
                output.push(if inside_quote { '“' } else { '”' });
            }
            _ => output.push(ch),
        }
    }

    output
}

fn xxreplace_quotes(text: &str) -> String {
    let mut output = String::new();
    let mut inside_quote = false;
    let mut inside_html_tag = false;

    for ch in text.chars() {
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
                // Handle single quotes logic
                output.push('’'); // Simplified for the example
            }
            '"' => {
                // Toggle inside_quote flag and handle double quotes
                inside_quote = !inside_quote;
                output.push(if inside_quote { '“' } else { '”' });
            }
            _ => output.push(ch),
        }
    }

    output
}

fn fix_punctuation(text: &str) -> String {
    let mut output = String::new();
    let chars: Vec<char> = text.chars().collect();
    let length = chars.len();
    let mut inside_quote = false;

    let mut i = 0;
    while i < length {
        let ch = chars[i];

        if ch == '<' || ch == '>' {
            output.push(ch);
        } else if ch == '"' || ch == '“' || ch == '”' {
            inside_quote = !inside_quote;
            output.push(ch);

            // Check if the next character is punctuation and we are closing a quote
            if !inside_quote && i + 1 < length && is_punctuation(chars[i + 1]) {
                i += 1; // Move past the quote
                output.push(chars[i]); // Add the punctuation inside the quote
            }
        } else {
            output.push(ch);
        }

        i += 1;
    }

    output
}

fn fix_em_anomaly(text: &str) -> String {
    let mut output = String::new();
    let chars: Vec<char> = text.chars().collect();
    let length = chars.len();

    let mut i = 0;
    while i < length {
        let ch = chars[i];

        if ch == '—' {
            // Remove space before em dash if it exists
            if !output.is_empty() && output.ends_with(' ') {
                output.pop();
            }

            output.push(ch);

            // Skip space after em dash if it exists
            if i + 1 < length && chars[i + 1] == ' ' {
                i += 1;
            }
        } else {
            output.push(ch);
        }

        i += 1;
    }

    output
}

fn fix_tshirt_anomaly(text: &str) -> String {
    let patterns = ["tshirt", "t-shirt", "Tshirt", "T-shirt"];
    let mut output = text.to_string();

    for pattern in &patterns {
        output = output.replace(pattern, "T-shirt");
    }

    output
}

fn xxreplace_quotes_and_fix_punctuation(text: &str) -> String {
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
