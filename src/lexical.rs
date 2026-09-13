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

fn extract_string(mut code: &str, mut line: usize) -> (&str, &str, usize) {
    let token_text;
    let mut search_index = 1;
    let mut end_index = code.len();

    while let Some(i) = (&code[search_index..]).find(['\"', '\r', '\n']) {
        end_index = search_index + i;

        // Se houver barra invertida antes, a string continua.
        if &code[end_index-1..end_index] == "\\" {
            search_index = end_index + 1;

            // Para o formato Unix de pular linha.
            if &code[end_index..end_index+1] == "\n" {
                line = line + 1;
            }
            // Para o formato Windows de pular linha.
            else if let Some("\r\n") = code.get(end_index..end_index+2) {
                search_index += 1;
                line = line + 1;
            }
        }
        // Se não, é o fim da string.
        else {
            if &code[end_index..end_index+1] == "\"" {
                end_index += 1;
            }

            break;
        }
    }

    (token_text, code) = code.split_at(end_index);
    
    return (token_text, code, line);
}

fn skip_comment(code: &str, mut line: usize) -> (usize, usize) {
    // Assume-se que já se consumiu um "(*" antes da função ser chamada.
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
                },

                "*" => {
                    if let Some(")") = code.get(index..index+1) {
                        delimiter_count -= 1;
                        index += 1;
                    }
                },

                "\n" => { line += 1; },
                &_ => ()
            }
        } else {
            index = code.len();
            break;
        }
    }
    
    return (index, line);
}

fn extract_token_text(mut code: &str, mut line: usize) -> (&str, &str, usize) {
    let token_text;
    code = code.trim_matches(WHITESPACES);
    
    if let Some(index) = code.find(SEPARATORS) {
        if index == 0 {
            match &code[0..1] {
                // String.
                "\"" => { return extract_string(code, line); },

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
                },

                // Comentário ou abre-parênteses.
                "(" => {
                    if let Some("*") = code.get(1..2) {
                        let index;
                        (index, line) = skip_comment(code, line);
                        return extract_token_text(&code[index..], line);
                    } else {
                        (token_text, code) = code.split_at(1);
                    }
                },

                // Operador de atribuição, de menor ou igual ou de menor.
                "<" => {
                    if let Some("-" | "=") = code.get(1..2) {
                        (token_text, code) = code.split_at(2);
                    } else {
                        (token_text, code) = code.split_at(1);
                    }
                },

                // Quebra de linha (Windows ou Unix).
                "\r" => { return extract_token_text(&code[1..], line); },
                "\n" => { return extract_token_text(&code[1..], line + 1); },

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

    return (token_text, code, line);
}

pub fn next_token(state: &mut State) {
    let tk_type;
    let tk_text;
    let tk_text_lowercase;

    (state.token_text, state.code, state.line) = extract_token_text(state.code, state.line);

    tk_text = state.token_text;
    tk_text_lowercase = tk_text.to_ascii_lowercase();
    
    if tk_text == "" {
        tk_type = TokenType::Eof;
    }
    else if &tk_text[0..1] == "\"" {
        let length = tk_text.len();
        
        if length >=2 && &tk_text[length-1..length] == "\"" {
            tk_type = TokenType::String;
        } else {
            tk_type = TokenType::Error;
        }
    }
    else if !tk_text.is_ascii() {
        tk_type = TokenType::Error;
    }
    else if tk_text.chars().all(char::is_numeric) {
        tk_type = TokenType::Integer;
    }
    else if DELIMITER.into_iter().any(|s| s == tk_text) {
        tk_type = TokenType::Delimiter;
    }
    else if BINARY_OPERATOR.into_iter().any(|s| s == tk_text) {
        tk_type = TokenType::BinaryOperator;
    }
    else if UNARY_OPERATOR.into_iter().any(|s| s == tk_text_lowercase) {
        tk_type = TokenType::UnaryOperator;
    }
    else if CONTROL.into_iter().any(|s| s == tk_text_lowercase) {
        tk_type = TokenType::Control;
    }
    else if DEFINITION.into_iter().any(|s| s == tk_text_lowercase) {
        tk_type = TokenType::Definition;
    }
    else if BOOLEAN.into_iter().any(|s| s == tk_text_lowercase) {
        if tk_text.starts_with(char::is_lowercase) {
            tk_type = TokenType::Boolean;
        } else {
            tk_type = TokenType::Error;
        }
    }
    else {
        let test = tk_text.replace('_', "");

        if test.chars().all(char::is_alphanumeric) {
            if tk_text.starts_with(char::is_uppercase) {
                tk_type = TokenType::TypeID;
            } else {
                tk_type = TokenType::ObjectID;
            }
        } else {
            tk_type = TokenType::Error;
        }
    }
    
    state.last_token = Token {
        tk_type: tk_type,
        value: tk_text.replace(['\r', '\n'], ""),
        line: state.line
    };
}
