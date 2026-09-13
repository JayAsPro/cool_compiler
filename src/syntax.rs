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
            "ERRO SINTÁTICO (linha {}): encontrou \"{}\".",
            token.line, token.value
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

fn optional_list(
    state: &mut State, var: fn(&mut State) -> bool, sep: &str, term: &str) -> bool {
    lexical::next_token(state);
        
    if state.last_token.value != term {
        if !var(state) { return false; }
        lexical::next_token(state);

        while state.last_token.value == sep {
            lexical::next_token(state);
            if !var(state) { return false; }
            lexical::next_token(state);
        }
    }
    
    return token_value(&state.last_token, term);
}

fn expr(state: &mut State) -> bool {
    lexical::next_token(state);

    if state.last_token.tk_type == TokenType::Integer
    || state.last_token.tk_type == TokenType::String
    || state.last_token.tk_type == TokenType::Boolean {
        lexical::next_token(state);
    }
    else if state.last_token.value == "new" {
        if !next_token_type(state, TokenType::TypeID) { return false; }
        lexical::next_token(state);
    }
    else if state.last_token.tk_type == TokenType::ObjectID {
        lexical::next_token(state);
        if state.last_token.value == "<-" { return expr(state); }
    }
    else {
        eprintln!(
            "ERRO SINTÁTICO (linha {}): esperava início de expressão; encontrou \"{}\".",
            state.last_token.line, state.last_token.value
        );

        return false;
    }
    
    return true;
}

fn formal(state: &mut State) -> bool {
    if !token_type(&state.last_token, TokenType::ObjectID) { return false; }
    if !next_token_value(state, ":") { return false; }
    if !next_token_type(state, TokenType::TypeID) { return false; }
    return true;
}

fn feature(state: &mut State) -> bool {
    if !token_type(&state.last_token, TokenType::ObjectID) { return false; }
    lexical::next_token(state);

    // Método.
    if state.last_token.value == "(" {
        if !optional_list(state, formal, ",", ")") { return false };
        if !token_value(&state.last_token, ")") { return false; }

        if !next_token_value(state, ":") { return false; }
        if !next_token_type(state, TokenType::TypeID) { return false; }

        if !next_token_value(state, "{") { return false; }
        if !expr(state) { return false; }
        if !token_value(&state.last_token, "}") { return false; }
        
        lexical::next_token(state);
    }
    // Atributo.
    else if state.last_token.value == ":" {
        if !next_token_type(state, TokenType::TypeID) { return false; }
        lexical::next_token(state);
        if state.last_token.value == "<-" { return expr(state); }
    }
    else {
        eprintln!(
            "ERRO SINTÁTICO (linha {}): esperava \"(\" ou \":\"; encontrou \"{}\"", 
            state.last_token.line, state.last_token.value
        );

        return false;
    }

    return true;
}

fn class(state: &mut State) -> bool {
    if !token_value(&state.last_token, "class") { return false; }
    if !next_token_type(state, TokenType::TypeID) { return false; }

    lexical::next_token(state);
    if state.last_token.value == "inherits" {
        if !next_token_type(state, TokenType::TypeID) { return false; }
        lexical::next_token(state);
    }

    if !token_value(&state.last_token, "{") { return false; }
    lexical::next_token(state);

    while state.last_token.value != "}" {
        if !feature(state) { return false; }
        if !token_value(&state.last_token, ";") { return false; }
        lexical::next_token(state);
    }
    
    return token_value(&state.last_token, "}");
}

pub fn program(code: &str) -> bool {
    let mut state = State::new(code);
    lexical::next_token(&mut state);

    while state.last_token.tk_type != TokenType::Eof {
        if !class(&mut state) { return false; }
        if !next_token_value(&mut state, ";") { return false; }
        lexical::next_token(&mut state);
    }

    return true;
}

/* 
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
*/
