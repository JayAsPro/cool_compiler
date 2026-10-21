use crate::common::*;
use crate::lexical;

#[derive(Debug)]
struct Attribute {
    object_id: String,
    type_id: String,
    expr: Expr
}

#[derive(Debug)]
struct Method {
    object_id: String,
    type_id: String,
    formals: Vec<Formal>,
    expr: Expr
}

#[derive(Debug)]
struct CaseOfBranch {
    object_id: String,
    type_id: String,
    expr: Expr
}

#[derive(Debug)]
enum Expr {
    Constant {
        type_id: String,
        value: String
    },

    Identifier {
        type_id: String,
        object_id: String
    },

    Assigment {
        type_id: String,
        object_id: String,
        expr: Box<Expr>
    },

    Dispatch {
        type_id: String,
        expr: Box<Expr>,
        class: String,
        object_id: String,
        params: Vec<Expr>
    },

    Conditional {
        type_id: String,
        expr_if: Box<Expr>,
        then: Box<Expr>,
        expr_else: Box<Expr>
    },

    Loop {
        type_id: String,
        expr_while: Box<Expr>,
        expr_loop: Box<Expr>
    },
    
    Block {
        type_id: String,
        list: Vec<Expr>
    },

    Let {
        type_id: String,
        list: Vec<Attribute>
    },

    Case {
        type_id: String,
        case: Box<Expr>,
        list: Vec<CaseOfBranch>
    },

    New {
        type_id: String
    },

    Isvoid {
        type_id: String,
        expr: Box<Expr>
    },

    BinaryOperator {
        type_id: String,
        expr1: Box<Expr>,
        op: String,
        expr2: Box<Expr>
    },

    UnaryOperator {
        type_id: String,
        op: String,
        expr: Box<Expr>
    },

    Nil
}

#[derive(Debug)]
struct Formal {
    object_id: String,
    type_id: String
}

#[derive(Debug)]
enum Feature {
    Attribute(Attribute),
    Method(Method)
}

#[derive(Debug)]
struct Class {
    type_id: String,
    inherits: String,
    attributes: Vec<Attribute>,
    methods: Vec<Method>
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

fn list_semicolon<T>(
    state: &mut State<Program>,
    variable: fn(&mut State<Program>) -> Result<T, T>,
    terminator: &str
) -> Result<Vec<T>, Vec<T>> {
    let mut list: Vec<T> = Vec::new();

    loop {
        match variable(state) {
            Ok(node) => { list.push(node); }
            Err(node) => { list.push(node); return Err(list) }
        }
        if !token_value(&state.last_token, ";") { return Err(list) }
        
        lexical::next_token(state);
        if state.last_token.value == terminator { return Ok(list) }
    }
}

fn list_semicolon_temp(
    state: &mut State<Program>, variable: fn(&mut State<Program>) -> bool, terminator: &str
) -> bool {
    loop {
        if !variable(state) { return false; }
        if !token_value(&state.last_token, ";") { return false; }
        
        lexical::next_token(state);
        if state.last_token.value == terminator { return true; }
    }
}

fn list_comma<T>(
    state: &mut State<Program>,
    variable: fn(&mut State<Program>) -> Result<T, T>,
    terminator: &str
) -> Result<Vec<T>, Vec<T>> {
    let mut list: Vec<T> = Vec::new();
    
    lexical::next_token(state);
        
    if state.last_token.value != terminator {
        match variable(state) {
            Ok(node) => { list.push(node); }
            Err(node) => { list.push(node); return Err(list) }
        }
        
        while state.last_token.value == "," {
            lexical::next_token(state);
            match variable(state) {
                Ok(node) => { list.push(node); }
                Err(node) => { list.push(node); return Err(list) }
            }
        }
    }
    
    if !token_value(&state.last_token, terminator) { return Err(list); }
    return Ok(list)
}

fn list_comma_temp(
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
                if !list_comma_temp(state, expr, ")") { return false; }
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
                    if !list_semicolon_temp(state, expr, "}") { return false; }
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
                    if !list_comma_temp(state, object_definition, "in") { return false; }
                    lexical::next_token(state);
                    if !expr(state) { return false; }
                }

                "case" => {
                    lexical::next_token(state);
                    if !expr(state) { return false; }

                    if !token_value(&state.last_token, "of") { return false; }
                    lexical::next_token(state);
                    if !list_semicolon_temp(state, case_of, "esac") { return false; }

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
                if !list_comma_temp(state, expr, ")") { return false; }

                lexical::next_token(state);
            }

            "." => {
                if !next_token_type(state, TokenType::ObjectID) { return false; }

                if !next_token_value(state, "(") { return false; }
                if !list_comma_temp(state, expr, ")") { return false; }
                
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

fn formal(state: &mut State<Program>) -> Result<Formal, Formal> {
    let mut formal = Formal {
        object_id: String::new(),
        type_id: String::new()
    };

    if !token_type(&state.last_token, TokenType::ObjectID) { return Err(formal) }
    formal.object_id = state.last_token.value.clone();

    if !next_token_value(state, ":") { return Err(formal) }
    if !next_token_type(state, TokenType::TypeID) { return Err(formal) }
    formal.type_id = state.last_token.value.clone();

    lexical::next_token(state);
    return Ok(formal);
}

fn feature(state: &mut State<Program>) -> Result<Feature, Feature> {
    let mut attribute = Attribute {
        object_id: String::new(),
        type_id: String::new(),
        expr: Expr::Nil
    };

    if !token_type(&state.last_token, TokenType::ObjectID) {
        return Err(Feature::Attribute(attribute));
    }

    let obj_id = state.last_token.value.clone();
    lexical::next_token(state);

    // Método.
    if state.last_token.value == "(" {
        let mut method = Method {
            object_id: obj_id,
            type_id: String::new(),
            formals: Vec::new(),
            expr: Expr::Nil
        };

        match list_comma(state, formal, ")") {
            Ok(list) => {method.formals = list;}
            Err(list) => { method.formals = list; return Err(Feature::Method(method)); }
        };
        
        if !next_token_value(state, ":") {
            return Err(Feature::Method(method))
        };
        if !next_token_type(state, TokenType::TypeID) {
            return Err(Feature::Method(method))
        };

        method.type_id = state.last_token.value.clone();

        if !next_token_value(state, "{") {
            return Err(Feature::Method(method))
        };

        lexical::next_token(state);

        if !expr(state) {
            return Err(Feature::Method(method))
        };

        if !token_value(&state.last_token, "}") {
            return Err(Feature::Method(method))
        };
        
        lexical::next_token(state);

        return Ok(Feature::Method(method));
    }
    // Atributo.
    else if state.last_token.value == ":" {
        attribute.object_id = obj_id;

        if !next_token_type(state, TokenType::TypeID) {
            return Err(Feature::Attribute(attribute))
        };
        lexical::next_token(state);

        if state.last_token.value == "<-" {
            lexical::next_token(state);
            if !expr(state) {
                return Err(Feature::Attribute(attribute))
            };
        }

        return Ok(Feature::Attribute(attribute));
    }
    else {
        eprintln!(
            "ERRO SINTÁTICO (linha {}): esperava \"(\" ou \":\"; encontrou \"{}\"", 
            state.last_token.line, state.last_token.value
        );

        return Err(Feature::Attribute(attribute));
    }
}

fn class(state: &mut State<Program>) -> Result<Class, Class> {
    let mut class = Class {
        type_id: String::new(),
        inherits: String::from("Object"),
        attributes: Vec::new(),
        methods: Vec::new()
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
        match list_semicolon(state, feature, "}") {
            Ok(list) => {
                for feature in list {
                    match feature {
                        Feature::Attribute(attribute) => { class.attributes.push(attribute) }
                        Feature::Method(method) => { class.methods.push(method) }
                    }
                }
            }
            Err(list) => {
                for feature in list {
                    match feature {
                        Feature::Attribute(attribute) => { class.attributes.push(attribute) }
                        Feature::Method(method) => { class.methods.push(method) }
                    }
                }
                return Err(class)
            }
        }
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
