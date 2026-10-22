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
        from_type_id: String,
        object_id: String,
        params: Vec<Expr>
    },

    Conditional {
        type_id: String,
        expr_if: Box<Expr>,
        expr_then: Box<Expr>,
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
        list: Vec<Attribute>,
        expr: Box<Expr>
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

    Parenthesis {
        type_id: String,
        expr: Box<Expr>
    },

    List {
        type_id: String,
        list: Vec<Expr>
    },

    Nil
}

impl Expr {
    fn type_id(&self) -> String {
        return match self {
            Expr::Constant { type_id, ..} => type_id,
            Expr::Identifier { type_id, ..} => type_id,
            Expr::Assigment { type_id, ..} => type_id,
            Expr::Dispatch { type_id, ..} => type_id,
            Expr::Conditional { type_id, ..} => type_id,
            Expr::Loop { type_id, ..} => type_id,
            Expr::Block { type_id, ..} => type_id,
            Expr::Let { type_id, ..} => type_id,
            Expr::Case { type_id, ..} => type_id,
            Expr::New { type_id, ..} => type_id,
            Expr::Isvoid { type_id, ..} => type_id,
            Expr::BinaryOperator { type_id, ..} => type_id,
            Expr::UnaryOperator { type_id, ..} => type_id,
            Expr::Parenthesis { type_id, ..} => type_id,
            Expr::List { type_id, ..} => type_id,
            Expr::Nil => ""
        }.to_string()
    }
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
            classes: vec![
                Class {
                    type_id: String::from("Object"),
                    inherits: String::new(),
                    attributes: Vec::new(),

                    methods: vec![
                        Method {
                            object_id: String::from("abort"),
                            type_id: String::from("Object"),
                            formals: Vec::new(),
                            expr: Expr::Nil
                        },

                        Method {
                            object_id: String::from("type_name"),
                            type_id: String::from("String"),
                            formals: Vec::new(),
                            expr: Expr::Nil
                        },

                        Method {
                            object_id: String::from("copy"),
                            type_id: String::from("SELF_TYPE"),
                            formals: Vec::new(),
                            expr: Expr::Nil
                        }
                    ]
                },

                Class {
                    type_id: String::from("Int"),
                    inherits: String::from("Object"),
                    attributes: Vec::new(),
                    methods: Vec::new()
                },

                Class {
                    type_id: String::from("Bool"),
                    inherits: String::from("Object"),
                    attributes: Vec::new(),
                    methods: Vec::new()
                },

                Class {
                    type_id: String::from("String"),
                    inherits: String::from("Object"),
                    attributes: Vec::new(),
                    methods: vec![
                        Method {
                            object_id: String::from("lenght"),
                            type_id: String::from("Int"),
                            formals: Vec::new(),
                            expr: Expr::Nil
                        },

                        Method {
                            object_id: String::from("concat"),
                            type_id: String::from("String"),
                            
                            formals: vec![
                                Formal {
                                    object_id: String::from("s"),
                                    type_id: String::from("String")
                                }
                            ],

                            expr: Expr::Nil
                        },

                        Method {
                            object_id: String::from("substr"),
                            type_id: String::from("SELF_TYPE"),

                            formals: vec![
                                Formal {
                                    object_id: String::from("i"),
                                    type_id: String::from("Int")
                                },

                                Formal {
                                    object_id: String::from("l"),
                                    type_id: String::from("Int")
                                }
                            ],

                            expr: Expr::Nil
                        }
                    ]
                },

                Class {
                    type_id: String::from("IO"),
                    inherits: String::from("Object"),
                    attributes: Vec::new(),

                    methods: vec![
                        Method {
                            object_id: String::from("out_string"),
                            type_id: String::from("SELF_TYPE"),

                            formals: vec![
                                Formal {
                                    object_id: String::from("x"),
                                    type_id: String::from("String")
                                }
                            ],

                            expr: Expr::Nil
                        },

                        Method {
                            object_id: String::from("out_int"),
                            type_id: String::from("SELF_TYPE"),

                            formals: vec![
                                Formal {
                                    object_id: String::from("x"),
                                    type_id: String::from("Int")
                                }
                            ],

                            expr: Expr::Nil
                        },

                        Method {
                            object_id: String::from("in_string"),
                            type_id: String::from("String"),
                            formals: Vec::new(),
                            expr: Expr::Nil
                        },

                        Method {
                            object_id: String::from("in_int"),
                            type_id: String::from("Int"),
                            formals: Vec::new(),
                            expr: Expr::Nil
                        }
                    ]
                }
            ]
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

fn type_and_parents<'a>(mut type_id: &'a str, classes: &'a Vec<Class>) -> Vec<&'a str> {
    let mut type_ids: Vec<&'a str> = Vec::new();

    for class in classes.iter().rev() {
        if class.type_id == type_id {
            type_ids.push(&class.type_id);
            type_id = &class.inherits;
        }
    }

    return type_ids;
}

fn erro_semantico<T: std::fmt::Debug>(node: &T, token: &Token, message: &str) {
    eprintln!("{:#?}", node);
    eprintln!("ERRO SEMÂNTICO (linha {}): {}.", token.line, message);
}

fn type_exists<'a>(mut type_id: &'a str, classes: &'a Vec<Class>) -> bool {
    return type_and_parents(type_id, &classes).is_empty()
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

fn object_definition(state: &mut State<Program>) -> Result<Attribute, Attribute> {
    let mut attribute = Attribute {
        object_id: String::new(),
        type_id: String::new(),
        expr: Expr::Nil
    };

    if !token_type(&state.last_token, TokenType::ObjectID) { return Err(attribute); }
    attribute.object_id = state.last_token.value.clone();

    if !next_token_value(state, ":") { return Err(attribute); }

    if !next_token_type(state, TokenType::TypeID) { return Err(attribute); }
    attribute.type_id = state.last_token.value.clone();

    lexical::next_token(state);
    if state.last_token.value == "<-" {
        lexical::next_token(state);

        match expr(state) {
            Ok(expr) => { attribute.expr = expr }
            Err(expr) => { attribute.expr = expr; return Err(attribute) }
        }
    }
    
    return Ok(attribute);
}

fn case_of(state: &mut State<Program>) -> Result<CaseOfBranch, CaseOfBranch> {
    let mut branch = CaseOfBranch {
        object_id: String::new(),
        type_id: String::new(),
        expr: Expr::Nil
    };

    if !token_type(&state.last_token, TokenType::ObjectID) { return Err(branch); }
    branch.object_id = state.last_token.value.clone();

    if !next_token_value(state, ":") { return Err(branch); }

    if !next_token_type(state, TokenType::TypeID) { return Err(branch); }
    branch.type_id = state.last_token.value.clone();

    lexical::next_token(state);
    
    if state.last_token.value == "=>" {
        lexical::next_token(state);

       match expr(state) {
            Ok(expr) => { branch.expr = expr }
            Err(expr) => { branch.expr = expr; return Err(branch) }
        }
    }
    
    return Ok(branch);
}

fn expr(state: &mut State<Program>) -> Result<Expr, Expr> {
    let mut expr_node;
    
    match state.last_token.tk_type {
        TokenType::Integer => {
            expr_node = Expr::Constant {
                type_id: String::from("Int"),
                value: state.last_token.value.clone()
            };
            
            lexical::next_token(state);
        }
        
        TokenType::String  => {
            expr_node = Expr::Constant {
                type_id: String::from("String"),
                value: state.last_token.value.clone()
            };
            
            lexical::next_token(state);
        }
        TokenType::Boolean => {
            expr_node = Expr::Constant {
                type_id: String::from("Bool"),
                value: state.last_token.value.clone()
            };
            
            lexical::next_token(state);
        }

        TokenType::ObjectID => {
            let obj_id = state.last_token.value.clone();
            lexical::next_token(state);

            if state.last_token.value == "<-" {
                let sub_expr;
                lexical::next_token(state);

                match expr(state) {
                    Ok(expr) => { sub_expr = expr }
                    Err(expr) => { return Err(Expr::Nil); }
                };

                expr_node = Expr::Assigment {
                    type_id: sub_expr.type_id(),
                    object_id: obj_id,
                    expr: Box::new(sub_expr)
                };
            }
            else if state.last_token.value == "(" {
                let params;

                match list_comma(state, expr, ")") {
                    Ok(list) => { params = list  }
                    Err(list) => { return Err(Expr::Nil) }
                }

                expr_node = Expr::Dispatch {
                    type_id: String::from(""),
                    
                    expr: Box::new(Expr::Identifier {
                        type_id: String::from("SELF_TYPE"),
                        object_id: String::from("self")
                    }),
                    
                    from_type_id: String::from("SELF_TYPE"),
                    object_id: obj_id,
                    params: params
                };
                
                lexical::next_token(state);
            } else {
                expr_node = Expr::Identifier {
                    type_id: String::from(""),
                    object_id: obj_id 
                }
            }
        }

        _ => {
            match state.last_token.value.as_str() {
                "new" => {
                    if !next_token_type(state, TokenType::TypeID) { return Err(Expr::Nil); }

                    expr_node = Expr::New {
                        type_id: state.last_token.value.clone()
                    };
                    
                    lexical::next_token(state);
                }

                "isvoid" => {
                    let sub_expr;
                    lexical::next_token(state);

                    match expr(state) {
                        Ok(expr) => { sub_expr = expr }
                        Err(expr) => { return Err(Expr::Nil); }
                    }

                    expr_node = Expr::Isvoid {
                        type_id: String::from("Bool"),
                        expr: Box::new(sub_expr)
                    };
                }

                "~" => {
                    let sub_expr;
                    lexical::next_token(state);

                    match expr(state) {
                        Ok(expr) => { sub_expr = expr }
                        Err(expr) => { return Err(Expr::Nil); }
                    }

                    expr_node = Expr::UnaryOperator {
                        type_id: String::from("Int"),
                        op: String::from("~"),
                        expr: Box::new(sub_expr)
                    };
                }

                "not" => {
                    let sub_expr;
                    lexical::next_token(state);

                    match expr(state) {
                        Ok(expr) => { sub_expr = expr }
                        Err(expr) => { return Err(Expr::Nil); }
                    }

                    expr_node = Expr::UnaryOperator {
                        type_id: String::from("Bool"),
                        op: String::from("not"),
                        expr: Box::new(sub_expr)
                    };
                }

                "(" => {
                    let sub_expr;
                    lexical::next_token(state);

                    match expr(state) {
                        Ok(expr) => { sub_expr = expr }
                        Err(expr) => { return Err(Expr::Nil); }
                    }

                    expr_node = Expr::Parenthesis {
                        type_id: sub_expr.type_id(),
                        expr: Box::new(sub_expr)
                    };
                    
                    if !token_value(&state.last_token, ")") { return Err(expr_node) }
                    lexical::next_token(state);
                }

                "{" => {
                    let expr_list;
                    lexical::next_token(state);

                    match list_semicolon(state, expr, "}") {
                        Ok(list) => { expr_list = list; }
                        Err(list) => { return Err(Expr::Nil) }
                    }

                    expr_node = Expr::List {
                        type_id: match expr_list.last() {
                            Some(expr) => expr.type_id(),
                            None => String::new()
                        },
                        
                        list: expr_list
                    };
                    
                    lexical::next_token(state);
                }

                "if" => {
                    let if_expr;
                    let then_expr;
                    let else_expr;

                    lexical::next_token(state);
                    
                    match expr(state) {
                        Ok(expr) => { if_expr = expr }
                        Err(expr) => { return Err(Expr::Nil); }
                    }
                    
                    if !token_value(&state.last_token, "then") { return Err(Expr::Nil); }
                    lexical::next_token(state);

                    match expr(state) {
                        Ok(expr) => { then_expr = expr }
                        Err(expr) => { return Err(Expr::Nil); }
                    }
                    
                    if !token_value(&state.last_token, "else") { return Err(Expr::Nil); }
                    lexical::next_token(state);

                    match expr(state) {
                        Ok(expr) => { else_expr = expr }
                        Err(expr) => { return Err(Expr::Nil); }
                    }

                    expr_node = Expr::Conditional {
                        type_id: String::from(""),
                        expr_if: Box::new(if_expr),
                        expr_then: Box::new(then_expr),
                        expr_else: Box::new(else_expr)
                    };

                    if !token_value(&state.last_token, "fi") { return Err(expr_node); }
                    lexical::next_token(state);
                }

                "while" => {
                    let while_expr;
                    let loop_expr;

                    lexical::next_token(state);
                    
                    match expr(state) {
                        Ok(expr) => { while_expr = expr }
                        Err(expr) => { return Err(Expr::Nil); }
                    }
                    
                    if !token_value(&state.last_token, "loop") { return Err(Expr::Nil); }
                    lexical::next_token(state);

                    match expr(state) {
                        Ok(expr) => { loop_expr = expr }
                        Err(expr) => { return Err(Expr::Nil); }
                    }

                    expr_node = Expr::Loop {
                        type_id: String::from("Object"),
                        expr_while: Box::new(while_expr),
                        expr_loop: Box::new(loop_expr)
                    };
                    
                    if !token_value(&state.last_token, "pool") { return Err(expr_node); }
                    lexical::next_token(state);
                }

                "let" => {
                    let attribs;
                    let sub_expr;

                    match list_comma(state, object_definition, "in") {
                        Ok(list) => { attribs = list}
                        Err(list) => { return Err(Expr::Nil)}
                    }

                    lexical::next_token(state);

                    match expr(state) {
                        Ok(expr) => { sub_expr = expr }
                        Err(expr) => { return Err(Expr::Nil); }
                    };

                    expr_node = Expr::Let {
                        type_id: sub_expr.type_id(),
                        list: attribs,
                        expr: Box::new(sub_expr)
                    };
                }

                "case" => {
                    let case;
                    let branches;

                    lexical::next_token(state);

                    match expr(state) {
                        Ok(expr) => { case = expr }
                        Err(expr) => { return Err(Expr::Nil); }
                    };

                    if !token_value(&state.last_token, "of") { return Err(Expr::Nil); }

                    lexical::next_token(state);

                    match list_semicolon(state, case_of, "esac") {
                        Ok(list) => { branches = list}
                        Err(list) => { return Err(Expr::Nil)}
                    }

                    lexical::next_token(state);

                    expr_node = Expr::Case {
                        type_id: String::from(""),
                        case: Box::new(Expr::Nil),
                        list: branches
                    };
                }

                _ => {
                    eprintln!(
                        "ERRO SINTÁTICO (linha {}): esperava início de expressão; encontrou \"{}\".",
                        state.last_token.line, state.last_token.value
                    );
                    
                    return Err(Expr::Nil);
                }
            }
        }
    }

    loop {
        match state.last_token.value.as_str() {
            "@" => {
                let from_type_id;
                let object_id;
                let params;
                
                if !next_token_type(state, TokenType::TypeID) { return Err(Expr::Nil); }
                from_type_id = state.last_token.value.clone();
                
                if !next_token_value(state, ".") { return Err(Expr::Nil); }
                
                if !next_token_type(state, TokenType::ObjectID) { return Err(Expr::Nil); }
                object_id = state.last_token.value.clone();

                if !next_token_value(state, "(") { return Err(Expr::Nil); }

                match list_comma(state, expr, ")") {
                    Ok(list) => { params = list  }
                    Err(list) => { return Err(Expr::Nil) }
                }

                lexical::next_token(state);

                expr_node = Expr::Dispatch {
                    type_id: String::from(""),
                    expr: Box::new(expr_node),
                    from_type_id: from_type_id,
                    object_id: object_id,
                    params: params
                };
            }

            "." => {
                let object_id;
                let from_type_id = expr_node.type_id();
                let params;
                
                if !next_token_type(state, TokenType::ObjectID) { return Err(Expr::Nil); }
                object_id = state.last_token.value.clone();

                if !next_token_value(state, "(") { return Err(Expr::Nil); }

                match list_comma(state, expr, ")") {
                    Ok(list) => { params = list  }
                    Err(list) => { return Err(Expr::Nil) }
                }
                
                lexical::next_token(state);

                expr_node = Expr::Dispatch {
                    type_id: String::from(""),
                    expr: Box::new(expr_node),
                    from_type_id: from_type_id,
                    object_id: object_id,
                    params: params
                };
            }

            "+" | "-" | "*" | "/" | "<" | "<=" | "=" => {
                let op = state.last_token.value.clone();
                let expr2;

                lexical::next_token(state);
                
                match expr(state) {
                    Ok(expr) => { expr2 = expr }
                    Err(expr) => { return Err(Expr::Nil); }
                };

                expr_node = Expr::BinaryOperator {
                    type_id: match op.as_str() {
                        "+" | "-" | "*" | "/" => String::from("Int"),
                        &_ => String::from("Bool")
                    },
                    
                    expr1: Box::new(expr_node),
                    op: op,
                    expr2: Box::new(expr2)
                };
            }
            
            &_ => { return Ok(expr_node); }
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

    if !type_exists(&formal.type_id, &state.tree.classes) {
        erro_semantico(&formal, &state.last_token, "tipo não definido");
        return Err(formal);
    }

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

        if !type_exists(&method.type_id, &state.tree.classes) {
            erro_semantico(&method, &state.last_token, "tipo não definido");
            return Err(Feature::Method(method));
        }

        if !next_token_value(state, "{") {
            return Err(Feature::Method(method))
        };

        lexical::next_token(state);
        
        match expr(state) {
            Ok(expr) => { method.expr = expr }
            Err(expr) => { method.expr = expr; return Err(Feature::Method(method)); }
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

        attribute.type_id = state.last_token.value.clone();

        if !type_exists(&attribute.type_id, &state.tree.classes) {
            erro_semantico(&attribute, &state.last_token, "tipo não definido");
            return Err(Feature::Attribute(attribute));
        }

        lexical::next_token(state);

        if state.last_token.value == "<-" {
            lexical::next_token(state);

            match expr(state) {
                Ok(expr) => { attribute.expr = expr }
                Err(expr) => { attribute.expr = expr; return Err(Feature::Attribute(attribute)); }
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

    if type_exists(&class.type_id, &state.tree.classes) {
        erro_semantico(&class, &state.last_token, "redefinição de tipo");
        return Err(class);
    }

    lexical::next_token(state);
    if state.last_token.value == "inherits" {
        if !next_token_type(state, TokenType::TypeID) { return Err(class); }
        class.inherits = state.last_token.value.clone();

        match class.inherits.as_str() {
            "Int" | "Bool" | "String" => {
                erro_semantico(&class, &state.last_token, "este tipo não pode ser herdado");
                return Err(class);
            }

            &_ => ()
        }
        
        if !type_exists(&class.type_id, &state.tree.classes) {
            erro_semantico(&class, &state.last_token, "tipo herdado não definido");
            return Err(class);
        }
        
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
            Ok(class) => { state.tree.classes.push(class) }

            Err(class) => {
                state.tree.classes.push(class);
                // println!("{:#?}", state.tree);
                return false;
            }
        }

        if !next_token_value(&mut state, ";") {
            // println!("{:#?}", state.tree);
            return false;
        }
        
        lexical::next_token(&mut state);
        if state.last_token.tk_type == TokenType::Eof {
            // println!("{:#?}", state.tree);
            return true;
        }
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
