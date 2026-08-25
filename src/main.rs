use std::env;
use std::fs;

const SEPARATORS: [char; 14] = [' ', ',', '.', ':', ';', '(', ')', '{', '}', '\"', '\\', '\t', '\r', '\n'];
const WHITESPACES: [char; 2] = [' ', '\t'];

const UNARY_OPERATOR: [&str; 3] = ["~", "not", "isvoid"];
const BINARY_OPERATOR: [&str; 9] = [".", "@", "+", "-", "*", "/", "=", "<", "<="];
const DELIMITER: [&str; 7] = [",", ":", ";", "(", ")", "{", "}"];
const CONTROL: [&str; 10] = ["if", "then", "else", "fi", "while", "loop", "pool", "case", "of", "esac"];
const DEFINITION: [&str; 5] = ["class", "inherits", "let", "in", "new"];
const BOOLEAN: [&str; 2] = ["true", "false"];

#[derive(Debug)]
enum TokenType {
    UnaryOperator,
    BinaryOperator,
    Delimiter,
    Boolean,
    Control,
    Definition,
    Name,
    Integer,
    String,
    Eof,
    Error
}

#[derive(Debug)]
struct Token {
    token_type: TokenType,
    value: String,
    line: u32
}

fn str_is_ascii_digits(string: &str) -> bool {
    for character in string.chars() {
        if !character.is_ascii_digit() {
            return false;
        }
    }

    return true;
}

fn multi_str_cmp(string: &str, string_array: &[&str]) -> bool {
    for &s in string_array {
        if string == s {
            return true;
        }
    }

    return false;
}

fn extract_token_text(mut code: &str, mut line_number: u32) -> (&str, &str, u32) {
    let token_text;
    code = code.trim_matches(WHITESPACES);
    
    if let Some(end_index) = code.find(SEPARATORS) {
        if end_index == 0 {
            match &code[0..1] {
                // Se for o início de uma string.
                "\"" => {
                    let mut search_index = 1;
                    let mut end_index = code.len();

                    while let Some(index) = (&code[search_index..]).find(['\"', '\r', '\n']) {
                        end_index = search_index + index;

                        // Lógica para strings divididas por várias linhas de código.
                        if &code[end_index-1..end_index] == "\\" {
                            // Para o formato Unix-like.
                            if &code[end_index..end_index+1] == "\n" {
                                search_index = end_index + 1;
                                line_number = line_number + 1;
                            }
                            // Para o formato Windows.
                            else if let Some("\r\n") = code.get(end_index..end_index+2) {
                                search_index = end_index + 2;
                                line_number = line_number + 1;
                            }
                        } else {
                            break;
                        }
                    }
                    
                    token_text = &code[0..end_index+1];
                    code = &code[end_index+1..];
                },
                "<" => {
                    if let Some("-" | "=") = code.get(1..2) {
                        token_text = &code[0..2];
                        code = &code[2..];
                    } else {
                        token_text = &code[0..1];
                        code = &code[1..];
                    }
                },

                // Se for um pular linha.
                "\r" => { return extract_token_text(&code[1..], line_number); } // Para o formato Windows. 
                "\n" => { return extract_token_text(&code[1..], line_number + 1); }
                &_ => { token_text = &code[0..1]; code = &code[1..]; }
            }
        } else {
            (token_text, code) = code.split_at(end_index);
        }
    } else {
        token_text = code;
        code = "";
    };

    return (token_text, code, line_number);
}

fn lexical_analysis(mut code: &str, mut line_number: u32) -> (Token, &str, u32) {
    let tk_type: TokenType;
    let tk_text;
    
    (tk_text, code, line_number) = extract_token_text(code, line_number);

    let tk_text_lowercase = &tk_text.to_ascii_lowercase();
    
    if tk_text == "" {
        tk_type = TokenType::Eof;
    } else if !tk_text.is_ascii() {
        tk_type = TokenType::Error;
    } else if str_is_ascii_digits(tk_text) {
        tk_type = TokenType::Integer;
    } else if multi_str_cmp(tk_text_lowercase, &UNARY_OPERATOR) {
        tk_type = TokenType::UnaryOperator;
    } else if multi_str_cmp(tk_text, &BINARY_OPERATOR) {
        tk_type = TokenType::BinaryOperator;
    } else if multi_str_cmp(tk_text, &DELIMITER) {
        tk_type = TokenType::Delimiter;
    } else if multi_str_cmp(tk_text_lowercase, &CONTROL) {
        tk_type = TokenType::Control;
    } else if multi_str_cmp(tk_text_lowercase, &DEFINITION) {
        tk_type = TokenType::Definition;
    } else if multi_str_cmp(tk_text_lowercase, &BOOLEAN) {
        tk_type = TokenType::Boolean;
    } else if &tk_text[0..1] == "\"" && &tk_text[tk_text.len()-1..tk_text.len()] == "\"" {
        tk_type = TokenType::String;
    } else {
        tk_type = TokenType::Name;
    }
    
    let token = Token {
        token_type: tk_type,
        value: tk_text.replace(['\r', '\n'], ""),
        line: line_number
    };
    
    (token, code, line_number)
}

fn syntax_analysis(mut code: &str) {
    let mut line_number: u32 = 1;

    while !code.is_empty() {
        let token;
        (token, code, line_number) = lexical_analysis(code, line_number);
        println!("{token:#?}");
    }
}

fn main() {
    let arguments: Vec<String> = env::args().collect();

    let Option::Some(file_path) = arguments.get(1) else {
        println!("ERRO: arquivo não informado!");
        return;
    };
    
    let Result::Ok(code) = fs::read_to_string(file_path) else {
        println!("ERRO: falha na abertura do arquivo!");
        return;
    };

    syntax_analysis(&code);
}
