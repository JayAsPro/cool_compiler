#[derive(Debug)]
pub struct Attribute {
    pub object_id: String,
    pub type_id: String,
    pub expr: Expr
}

#[derive(Debug)]
pub struct Method {
    pub object_id: String,
    pub type_id: String,
    pub formals: Vec<Formal>,
    pub expr: Expr
}

#[derive(Debug)]
pub struct CaseOfBranch {
    pub object_id: String,
    pub type_id: String,
    pub expr: Expr
}

#[derive(Debug)]
pub enum Expr {
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

    Nil
}

impl Expr {
    pub fn type_id(&self) -> String {
        return match self {
              Expr::Constant      { type_id, ..} | Expr::Identifier     { type_id, ..}
            | Expr::Assigment     { type_id, ..} | Expr::Dispatch       { type_id, ..}
            | Expr::Conditional   { type_id, ..} | Expr::Loop           { type_id, ..}
            | Expr::Block         { type_id, ..} | Expr::Let            { type_id, ..}
            | Expr::Case          { type_id, ..} | Expr::New            { type_id, ..}
            | Expr::Isvoid        { type_id, ..} | Expr::BinaryOperator { type_id, ..}
            | Expr::UnaryOperator { type_id, ..} | Expr::Parenthesis    { type_id, ..} => type_id,
            
            Expr::Nil => ""
        }.to_string()
    }
}

#[derive(Debug)]
pub struct Formal {
    pub object_id: String,
    pub type_id: String
}

#[derive(Debug)]
pub enum Feature {
    Attribute(Attribute),
    Method(Method)
}

#[derive(Debug)]
pub struct Class {
    pub type_id: String,
    pub inherits: String,
    pub attributes: Vec<Attribute>,
    pub methods: Vec<Method>
}

#[derive(Debug)]
pub struct Program {
    pub classes: Vec<Class>
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
