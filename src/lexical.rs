use crate::common::*;

const SEPARATORS: [char; 23] = [
    ' ', ',', '.', ':', ';', '~', '@', '+', '-', '*', '/', '=',
    '<', '>', '(', ')', '{', '}', '\\', '\"', '\t', '\r', '\n'
];

const WHITESPACES: [char; 4] = [' ', '\t', '\x0B', '\x0C'];

const DELIMITER: [&str; 7] = [",", ":", ";", "(", ")", "{", "}"];
const BINARY_OPERATOR: [&str; 10] = [".", "@", "<-", "=", "<", "<=", "+", "-", "*", "/"];
const UNARY_OPERATOR: [&str; 3] = ["~", "not", "isvoid"];

const CONTROL: [&str; 11] = [
    "if", "then", "else", "fi", "while", "loop", "pool", "case", "of", "esac", "=>"
];

const DEFINITION: [&str; 5] = ["class", "inherits", "let", "in", "new"];
const BOOLEAN: [&str; 2] = ["true", "false"];

fn extract_string(mut code: &str, mut line: usize) -> (String, &str, usize) {
    let mut token_text = String::from("\"");

    while let Some(index) = code.find(['\"', '\\', '\r', '\n']) {
        token_text += &code[..index];

        match &code[index..index+1] {
            "\\" => {
                code = &code[index+1..];

                match code.get(index..index+1) {
                    Some(text) => {
                        code = &code[index+1..];
                        
                        token_text += match text {
                            "r" => "\r",
                            "n" => "\n",
                            "t" => "\t",
                            _ => text
                        }
                    }
                    
                    _ => ()
                }

                if let Some("\r") = code.get(index..index+1) {
                    code = &code[index+1..];
                }

                if let Some("\n") = code.get(index..index+1) {
                    code = &code[index+1..].trim_matches(WHITESPACES);
                    line = line + 1;
                }
            }

            "\"" => {
                token_text += "\"";
                code = &code[index+1..];
                break;
            }

            &_ => {
                code = &code[index..];
                break;
            }
        }
    }

    return (String::from(token_text), code, line);
}

fn skip_comment(code: &str, mut line: usize) -> (usize, usize) {
    let mut delimiter_count = 1;
    let mut index = 2;

    while delimiter_count != 0 {
        if let Some(i) = (&code[index..]).find(['(', '*', '\r', '\n']) {
            index += i + 1;
            
            match &code[index-1..index] {
                "(" => {
                    if let Some("*") = code.get(index..index+1) {
                        delimiter_count += 1;
                        index += 1;
                    }
                }

                "*" => {
                    if let Some(")") = code.get(index..index+1) {
                        delimiter_count -= 1;
                        index += 1;
                    }
                }

                "\n" => { line += 1; }
                &_ => ()
            }
        } else {
            index = code.len();
            break;
        }
    }
    
    return (index, line);
}

fn extract_token_text(mut code: &str, mut line: usize) -> (String, &str, usize) {
    let token_text;
    code = code.trim_matches(WHITESPACES);
    
    if let Some(index) = code.find(SEPARATORS) {
        if index == 0 {
            match &code[0..1] {
                // String.
                "\"" => { return extract_string(&code[1..], line); }

                // Comentário de uma linha ou operador de subtração.
                "-" => {
                    if let Some("-") = code.get(1..2) {
                        if let Some(index) = code.find(['\r', '\n']) {
                            return extract_token_text(&code[index..], line);
                        } else {
                            token_text = "";
                            code = "";
                        }
                    } else {
                        (token_text, code) = code.split_at(1);
                    }
                }

                // Comentário ou abre-parênteses.
                "(" => {
                    if let Some("*") = code.get(1..2) {
                        let index;
                        (index, line) = skip_comment(code, line);
                        return extract_token_text(&code[index..], line);
                    } else {
                        (token_text, code) = code.split_at(1);
                    }
                }

                // Operador de atribuição, de menor ou igual ou de menor.
                "<" => {
                    if let Some("-" | "=") = code.get(1..2) {
                        (token_text, code) = code.split_at(2);
                    } else {
                        (token_text, code) = code.split_at(1);
                    }
                }

                // Operador de igual ou seta do "case of".
                "=" => {
                    if let Some(">") = code.get(1..2) {
                        (token_text, code) = code.split_at(2);
                    } else {
                        (token_text, code) = code.split_at(1);
                    }
                }

                // Quebra de linha (Windows ou Unix).
                "\r" => { return extract_token_text(&code[1..], line); }
                "\n" => { return extract_token_text(&code[1..], line + 1); }

                // Tokens de um caractere.
                &_ => { (token_text, code) = code.split_at(1); }
            }
        } else {
            (token_text, code) = code.split_at(index);
        }
    } else {
        token_text = code;
        code = "";
    };

    return (String::from(token_text), code, line);
}

pub fn next_token(state: &mut State) {
    (state.last_token.value,
     state.next_code,
     state.last_token.line) = extract_token_text(state.next_code, state.last_token.line);
    
    let value = &state.last_token.value;
    let value_lowercase = value.to_ascii_lowercase();
    let tk_type;
    
    if value == "" {
        tk_type = TokenType::Eof;
    }
    else if let Some("\"") = value.get(0..1) {
        let len = value.len();
        
        if let Some("\"") = value.get(len-1..len) && len >= 2 {
            tk_type = TokenType::String;
        } else {
            tk_type = TokenType::Error;
        }
    }
    else if !value.is_ascii() {
        tk_type = TokenType::Error;
    }
    else if value.chars().all(char::is_numeric) {
        tk_type = TokenType::Integer;
    }
    else if DELIMITER.into_iter().any(|s| s == value) {
        tk_type = TokenType::Delimiter;
    }
    else if BINARY_OPERATOR.into_iter().any(|s| s == value) {
        tk_type = TokenType::BinaryOperator;
    }
    else if UNARY_OPERATOR.into_iter().any(|s| s == value_lowercase) {
        state.last_token.value = value_lowercase;
        tk_type = TokenType::UnaryOperator;
    }
    else if CONTROL.into_iter().any(|s| s == value_lowercase) {
        state.last_token.value = value_lowercase;
        tk_type = TokenType::Control;
    }
    else if DEFINITION.into_iter().any(|s| s == value_lowercase) {
        state.last_token.value = value_lowercase;
        tk_type = TokenType::Definition;
    }
    else if BOOLEAN.into_iter().any(|s| s == value_lowercase) {
        if value.starts_with(char::is_lowercase) {
            tk_type = TokenType::Boolean;
        } else {
            tk_type = TokenType::Error;
        }
    }
    else {
        let test = value.replace('_', "");

        if test.chars().all(char::is_alphanumeric) {
            if value.starts_with(char::is_uppercase) {
                tk_type = TokenType::TypeID;
            } else {
                tk_type = TokenType::ObjectID;
            }
        } else {
            tk_type = TokenType::Error;
        }
    }

    state.last_token.tk_type = tk_type;

    println!("{:#?}", state.last_token);
}
