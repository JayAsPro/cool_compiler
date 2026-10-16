#[derive(Debug)]
#[derive(PartialEq)]
pub enum TokenType {
    Start,
    Delimiter,
    BinaryOperator,
    UnaryOperator,
    Control,
    Definition,
    Boolean,
    TypeID,
    ObjectID,
    Integer,
    String,
    Eof,
    Error
}

#[derive(Debug)]
pub struct Token {
    pub tk_type: TokenType,
    pub value: String,
    pub line: usize
}

impl Token {
    pub fn new() -> Token {
        return Token {
            tk_type: TokenType::Start,
            value: String::new(),
            line: 1
        };
    }
}

pub struct State<'a, T: Default> {
    pub last_token: Token,
    pub tree: T,
    pub next_code: &'a str,
}

impl<T: Default> State<'_, T> {
    pub fn new(code: &str) -> State<'_, T> {
        return State {
            last_token: Token::new(),
            tree: T::default(),
            next_code: code,
        };
    }
}
