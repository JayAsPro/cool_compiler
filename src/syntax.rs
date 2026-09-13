use crate::common::*;
use crate::lexical;

fn token_value(token: &Token, string: &str) -> bool {
    if token.value.to_ascii_lowercase() != string {
        eprintln!(
            "ERRO SINTÁTICO (linha {}): esperava \"{}\"; encontrou \"{}\".",
            token.line, string, token.value
        );

        return false;
    }
    
    return true;
}

fn token_type(token: &Token, tk_type: TokenType) -> bool {
    if token.tk_type != tk_type {
        eprintln!(
            "ERRO SINTÁTICO (linha {}).",
            token.line
        );
        return false;
    }
    
    return true;
}

fn next_token_value(state: &mut State, string: &str) -> bool {
    lexical::next_token(state);
    return token_value(&state.last_token, string);
}

fn next_token_type(state: &mut State, tk_type: TokenType) -> bool {
    lexical::next_token(state);
    return token_type(&state.last_token, tk_type);
}

/*
fn formal(mut code: &str, mut line_number: usize) -> (&str, bool, usize) {
    let mut token;

    (token, code, line_number) = lexical::next_token(code, line_number);
    println!("{token:#?}");

    if token.token_type != lexical::TokenType::TypeID {
        eprintln!(
            "ERRO SINTÁTICO na linha {line_number}: \
            esperava um TypeID."
        );

        return (code, false, line_number);
    }

    (token, code, line_number) = lexical::next_token(code, line_number);
    println!("{token:#?}");

    if token.value != ":" {
        eprintln!(
            "ERRO SINTÁTICO na linha {line_number}: \
            esperava um \":\"."
        );

        return (code, false, line_number);
    }

    (token, code, line_number) = lexical::next_token(code, line_number);
    println!("{token:#?}");

    if token.token_type != lexical::TokenType::ObjectID {
        eprintln!(
            "ERRO SINTÁTICO na linha {line_number}: \
            esperava um ObjectID."
        );

        return (code, false, line_number);
    }

    return (code, true, line_number);
}

*/

fn feature(state: &mut State) -> bool {
    if !next_token_type(state, TokenType::ObjectID) { return false; }

    lexical::next_token(state);

    if state.last_token.value == "(" {
        // if !formal(state) { return false; }
        // Loop de formal e ,.
        if !next_token_value(state, ")") { return false; }

        if !next_token_value(state, ":") { return false; }
        if !next_token_type(state, TokenType::TypeID) { return false; }

        if !next_token_value(state, "{") { return false; }
        // expr
        if !next_token_value(state, "}") { return false; }
    } else if state.last_token.value == ":" {
        if !next_token_type(state, TokenType::TypeID) { return false; }
        // Opcional: atribuição e expressão.
    } else {
        eprintln!("ERRO SINTÁTICO na linha {}: esperava.", state.last_token.line);
        return false;
    }

    return true;
}

fn class(state: &mut State) -> bool {
    if !next_token_value(state, "class") { return false; }
    if !next_token_type(state, TokenType::TypeID) { return false; }

    lexical::next_token(state);
    if state.last_token.value == "inherits" {
        if !next_token_type(state, TokenType::TypeID) { return false; }
        lexical::next_token(state);
    }

    if !token_value(&state.last_token, "{") { return false; }
    if !feature(state) { return false; }
    if !next_token_value(state, ";") { return false; }
    if !next_token_value(state, "}") { return false; }
    
    return true;
}


pub fn program(code: &str) -> bool {
    let mut state = State::new(code);

    while state.last_token.tk_type != TokenType::Eof {
        if !class(&mut state) { return false; }
        if !next_token_value(&mut state, ";") { return false; }
    }

    return true;
}

pub fn token_loop(code: &str) {
    let mut state = State::new(code);

    while state.last_token.tk_type != TokenType::Eof {
        lexical::next_token(&mut state);

        match state.last_token.tk_type {
            TokenType::Error => { eprintln!("ERRO: token inválido!"); break; },
            _ => ()
        }
    }
}
