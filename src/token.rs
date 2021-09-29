#[derive(Clone, Debug)]
pub struct Token<'a> {
    pub kind: TokenKind<'a>,
    pub line: u32,
    pub col: u32,
    pub span: &'a str,
}

impl<'a> Token<'a> {
    pub fn new(kind: TokenKind<'a>, line: u32, col: u32, span: &'a str) -> Self {
        Token { kind, line, col, span }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum TokenKind<'a> {
    LParen,
    RParen,
    LBrace,
    RBrace,
    Comma,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,
    Dollar,

    PlusEqual,
    MinusEqual,
    MultiplyEqual,
    DivideEqual,

    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    Ident(&'a str),
    Number(f64),

    Else,
    False,
    If,
    True,
}

impl<'a> TokenKind<'a> {
    pub fn ident(ident: &'a str) -> Self {
        match ident {
            "else" => TokenKind::Else,
            "false" => TokenKind::False,
            "if" => TokenKind::If,
            "true" => TokenKind::True,
            ident => TokenKind::Ident(ident),
        }
    }
}