use crate::common::*;
use crate::lexical;

#[derive(Debug)]
struct Formal {
    object_id: String,
    type_id: String
}

#[derive(Debug)]
enum Feature {
    Attribute { object_id: String, type_id: String },
    Method { object_id: String, type_id: String, formals: Vec<Formal> }
}

#[derive(Debug)]
struct Class {
    type_id: String,
    inherits: String,
    features: Vec<Feature>
}

#[derive(Debug)]
struct Program {
    classes: Vec<Class>
}

impl Default for Program {
    fn default() -> Self {
        Program {
            classes: Vec::new()
        }
    }
}

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

fn next_token_value(state: &mut State<Program>, string: &str) -> bool {
    lexical::next_token(state);
    return token_value(&state.last_token, string);
}

fn next_token_type(state: &mut State<Program>, tk_type: TokenType) -> bool {
    lexical::next_token(state);
    return token_type(&state.last_token, tk_type);
}

fn list_semicolon(
    state: &mut State<Program>, variable: fn(&mut State<Program>) -> bool, terminator: &str
) -> bool {
    loop {
        if !variable(state) { return false; }
        if !token_value(&state.last_token, ";") { return false; }
        
        lexical::next_token(state);
        if state.last_token.value == terminator { return true; }
    }
}

fn list_comma(
    state: &mut State<Program>, variable: fn(&mut State<Program>) -> bool, terminator: &str
) -> bool {
    lexical::next_token(state);
        
    if state.last_token.value != terminator {
        if !variable(state) { return false; }
        
        while state.last_token.value == "," {
            lexical::next_token(state);
            if !variable(state) { return false; }
        }
    }
    
    return token_value(&state.last_token, terminator);
}

fn object_definition(state: &mut State<Program>) -> bool {
    if !token_type(&state.last_token, TokenType::ObjectID) { return false; }
    if !next_token_value(state, ":") { return false; }
    if !next_token_type(state, TokenType::TypeID) { return false; }

    lexical::next_token(state);
    if state.last_token.value == "<-" {
        lexical::next_token(state);
        return expr(state);
    }
    
    return true;
}

fn case_of(state: &mut State<Program>) -> bool {
    if !token_type(&state.last_token, TokenType::ObjectID) { return false; }
    if !next_token_value(state, ":") { return false; }
    if !next_token_type(state, TokenType::TypeID) { return false; }
    lexical::next_token(state);
    
    if state.last_token.value == "=>" {
        lexical::next_token(state);
        return expr(state);
    }
    
    return true;
}

fn expr(state: &mut State<Program>) -> bool {
    match state.last_token.tk_type {
        TokenType::Integer => { lexical::next_token(state); }
        TokenType::String  => { lexical::next_token(state); }
        TokenType::Boolean => { lexical::next_token(state); }

        TokenType::ObjectID => {
            lexical::next_token(state);

            if state.last_token.value == "<-" {
                lexical::next_token(state);
                if !expr(state) { return false; }
            }
            else if state.last_token.value == "(" {
                if !list_comma(state, expr, ")") { return false; }
                lexical::next_token(state);
            }
        }

        _ => {
            match state.last_token.value.as_str() {
                "new" => {
                    if !next_token_type(state, TokenType::TypeID) { return false; }
                    lexical::next_token(state);
                }

                "isvoid" => {
                    lexical::next_token(state);
                    if !expr(state) { return false; }
                }

                "~" => {
                    lexical::next_token(state);
                    if !expr(state) { return false; }
                }

                "not" => {
                    lexical::next_token(state);
                    if !expr(state) { return false; }
                }

                "(" => {
                    lexical::next_token(state);
                    if !expr(state) { return false; }
                    if !token_value(&state.last_token, ")") { return false; }
                    lexical::next_token(state);
                }

                "{" => {
                    lexical::next_token(state);
                    if !list_semicolon(state, expr, "}") { return false; }
                    lexical::next_token(state);
                }

                "if" => {
                    lexical::next_token(state);
                    if !expr(state) { return false; }
                    
                    if !token_value(&state.last_token, "then") { return false; }
                    lexical::next_token(state);
                    if !expr(state) { return false; }
                    
                    if !token_value(&state.last_token, "else") { return false; }
                    lexical::next_token(state);
                    if !expr(state) { return false; }

                    if !token_value(&state.last_token, "fi") { return false; }
                    lexical::next_token(state);
                }

                "while" => {
                    lexical::next_token(state);
                    if !expr(state) { return false; }
                    
                    if !token_value(&state.last_token, "loop") { return false; }
                    lexical::next_token(state);
                    if !expr(state) { return false; }
                    
                    if !token_value(&state.last_token, "pool") { return false; }
                    lexical::next_token(state);
                }

                "let" => {
                    if !list_comma(state, object_definition, "in") { return false; }
                    lexical::next_token(state);
                    if !expr(state) { return false; }
                }

                "case" => {
                    lexical::next_token(state);
                    if !expr(state) { return false; }

                    if !token_value(&state.last_token, "of") { return false; }
                    lexical::next_token(state);
                    if !list_semicolon(state, case_of, "esac") { return false; }

                    lexical::next_token(state);
                }

                _ => {
                    eprintln!(
                        "ERRO SINTÁTICO (linha {}): esperava início de expressão; encontrou \"{}\".",
                        state.last_token.line, state.last_token.value
                    );

                    return false;
                }
            }
        }
    }

    loop {
        match state.last_token.value.as_str() {
            "@" => {
                if !next_token_type(state, TokenType::TypeID) { return false; }
                if !next_token_value(state, ".") { return false; }
                if !next_token_type(state, TokenType::ObjectID) { return false; }

                if !next_token_value(state, "(") { return false; }
                if !list_comma(state, expr, ")") { return false; }

                lexical::next_token(state);
            }

            "." => {
                if !next_token_type(state, TokenType::ObjectID) { return false; }

                if !next_token_value(state, "(") { return false; }
                if !list_comma(state, expr, ")") { return false; }
                
                lexical::next_token(state);
            }

            "+" | "-" | "*" | "/" | "<" | "<=" | "=" => {
                lexical::next_token(state);
                if !expr(state) { return false; }
            }
            
            &_ => { return true; }
        }
    }
}

fn formal(state: &mut State<Program>) -> bool {
    if !token_type(&state.last_token, TokenType::ObjectID) { return false; }
    if !next_token_value(state, ":") { return false; }
    if !next_token_type(state, TokenType::TypeID) { return false; }

    lexical::next_token(state);
    return true;
}

fn feature(state: &mut State<Program>) -> bool {
    if !token_type(&state.last_token, TokenType::ObjectID) { return false; }
    lexical::next_token(state);

    // Método.
    if state.last_token.value == "(" {
        if !list_comma(state, formal, ")") { return false };
        
        if !next_token_value(state, ":") { return false; }
        if !next_token_type(state, TokenType::TypeID) { return false; }

        if !next_token_value(state, "{") { return false; }
        lexical::next_token(state);
        if !expr(state) { return false; }
        if !token_value(&state.last_token, "}") { return false; }
        
        lexical::next_token(state);
    }
    // Atributo.
    else if state.last_token.value == ":" {
        if !next_token_type(state, TokenType::TypeID) { return false; }
        lexical::next_token(state);

        if state.last_token.value == "<-" {
            lexical::next_token(state);
            return expr(state);
        }
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

fn class(state: &mut State<Program>) -> Result<Class, Class> {
    let mut class = Class {
        type_id: String::new(),
        inherits: String::from("Object"),
        features: Vec::new()
    };
    
    if !token_value(&state.last_token, "class") { return Err(class); }
    if !next_token_type(state, TokenType::TypeID) { return Err(class); }
    class.type_id.push_str(&state.last_token.value);

    lexical::next_token(state);
    if state.last_token.value == "inherits" {
        if !next_token_type(state, TokenType::TypeID) { return Err(class); }
        class.inherits = state.last_token.value.clone();
        lexical::next_token(state);
    }

    if !token_value(&state.last_token, "{") { return Err(class); }
    lexical::next_token(state);

    if state.last_token.value != "}" {
        if !list_semicolon(state, feature, "}") {return Err(class)}
    } else {
        if !token_value(&state.last_token, "}") { return Err(class); }
    }

    return Ok(class);
}

pub fn program(code: &str) -> bool {
    let mut state = State::<Program>::new(code);
    lexical::next_token(&mut state);

    loop {
        match class(&mut state) {
            Ok(class) => { state.tree.classes.push(class)}
            Err(class) => { state.tree.classes.push(class); return false; }
        }

        if !next_token_value(&mut state, ";") { return false; }
        
        lexical::next_token(&mut state);
        if state.last_token.tk_type == TokenType::Eof { println!("{:#?}", state.tree); return true; }
    }
}

/* 
pub fn token_loop(code: &str) {
    let mut state = State<Program>::new(code);

    while state.last_token.tk_type != TokenType::Eof {
        lexical::next_token(&mut state);

        match state.last_token.tk_type {
            TokenType::Error => { eprintln!("ERRO: token inválido!"); break; },
            _ => ()
        }
    }
}
*/
