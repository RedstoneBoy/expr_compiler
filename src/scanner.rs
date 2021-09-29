use std::iter::Peekable;
use std::str::Chars;

use crate::token::{Token, TokenKind};

const TAB_SIZE: u32 = 4;

pub struct Scanner<'a> {
    source: &'a str,
    chars: Peekable<Chars<'a>>,

    start: usize,
    current: usize,

    line: u32,
    col: u32,

    cur_line: u32,
    cur_col: u32,
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a str) -> Self {
        Scanner {
            source,
            chars: source.chars().peekable(),

            start: 0,
            current: 0,

            line: 1,
            col: 1,

            cur_line: 1,
            cur_col: 1,
        }
    }

    pub fn scan(mut self) -> Vec<Result<Token<'a>, Error>> {
        let mut vec = Vec::new();

        loop {
            if self.chars.peek() == None {
                break;
            }

            vec.push(self.scan_token());
        }

        vec
    }

    pub fn scan_token(&mut self) -> Result<Token<'a>, Error> {
        self.skip_whitespace();

        self.start = self.current;
        self.cur_line = self.line;
        self.cur_col = self.col;

        let ch = self.advance_err()?;
        
        let token = match ch {
            '(' => self.make_token(TokenKind::LParen),
            ')' => self.make_token(TokenKind::RParen),
            '{' => self.make_token(TokenKind::LBrace),
            '}' => self.make_token(TokenKind::RBrace),
            ';' => self.make_token(TokenKind::Semicolon),
            ',' => self.make_token(TokenKind::Comma),
            '-' => self.make_double_token('=', TokenKind::Minus, TokenKind::MinusEqual)?,
            '+' => self.make_double_token('=', TokenKind::Plus, TokenKind::PlusEqual)?,
            '/' => self.make_double_token('=', TokenKind::Slash, TokenKind::DivideEqual)?,
            '*' => self.make_double_token('=', TokenKind::Star, TokenKind::MultiplyEqual)?,
            '$' => self.make_token(TokenKind::Dollar),
            '!' => self.make_double_token('=', TokenKind::Bang, TokenKind::BangEqual)?,
            '=' => self.make_double_token('=', TokenKind::Equal, TokenKind::EqualEqual)?,
            '<' => self.make_double_token('=', TokenKind::Less, TokenKind::LessEqual)?,
            '>' => self.make_double_token('=', TokenKind::Greater, TokenKind::GreaterEqual)?,
            ch if is_digit(ch) => self.parse_number()?,
            ch if is_alpha(ch) => self.parse_ident()?,
            ch => return Err(self.make_error(ErrorKind::UnexpectedCharacter, ch.to_string())),
        };

        Ok(token)
    }

    fn advance_err(&mut self) -> Result<char, Error> {
        self.advance().ok_or_else(|| self.make_error(ErrorKind::UnexpectedEof, "unexpected eof".to_owned()))
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.chars.next()?;
        self.current += ch.len_utf8();

        match ch {
            '\t' => self.col += TAB_SIZE,
            '\r' => (),
            '\n' => {
                self.col = 1;
                self.line += 1;
            }
            _ => self.col += 1,
        }

        Some(ch)
    }

    fn parse_ident(&mut self) -> Result<Token<'a>, Error> {
        loop {
            match self.chars.peek() {
                Some(&ch) if is_alpha(ch) || is_digit(ch) => {
                    self.advance_err()?;
                }
                _ => break,
            }
        }

        Ok(self.make_ident())
    }

    fn parse_number(&mut self) -> Result<Token<'a>, Error> {
        loop {
            match self.chars.peek() {
                Some(&ch) if is_digit(ch) => {
                    self.advance_err()?;
                }
                _ => break,
            }
        }

        if self.chars.peek() == Some(&'.') {
            self.advance_err()?;

            loop {
                match self.chars.peek() {
                    Some(&ch) if is_digit(ch) => {
                        self.advance_err()?;
                    }
                    _ => break,
                }
            }
        }

        Ok(self.make_number())
    }

    fn skip_whitespace(&mut self) {
        loop {
            match self.chars.peek() {
                Some(' ')
                    | Some('\r')
                    | Some('\t')
                    | Some('\n') => { self.advance(); }
                _ => break,
            }
        }
    }

    fn get_text(&self) -> &'a str {
        &self.source[self.start..self.current]
    }

    fn make_token(&self, kind: TokenKind<'a>) -> Token<'a> {
        let text = self.get_text();
        
        Token::new(kind, self.cur_line, self.cur_col, text)
    }

    fn make_double_token(&mut self, second_char: char, default: TokenKind<'a>, double: TokenKind<'a>) -> Result<Token<'a>, Error> {
        if self.chars.peek() == Some(&second_char) {
            self.advance_err()?;
            Ok(self.make_token(double))
        } else {
            Ok(self.make_token(default))
        }
    }

    fn make_ident(&self) -> Token<'a> {
        let text = self.get_text();

        self.make_token(TokenKind::ident(text))
    }

    fn make_number(&self) -> Token<'a> {
        let text = self.get_text();

        // This should only be called with a valid float literal
        let float = text.parse::<f64>().unwrap();

        self.make_token(TokenKind::Number(float))
    }

    fn make_error(&self, kind: ErrorKind, msg: String) -> Error {
        Error::new(kind, msg, self.line, self.col)
    }
}

fn is_digit(ch: char) -> bool {
    ch.is_digit(10)
}

fn is_alpha(ch: char) -> bool {
    ch.is_alphabetic() || ch == '_'
}

#[derive(Debug)]
pub struct Error {
    pub kind: ErrorKind,
    pub msg: String,
    pub line: u32,
    pub col: u32,
}

impl Error {
    pub fn new(kind: ErrorKind, msg: String, line: u32, col: u32) -> Self {
        Error { kind, msg, line, col }
    }
}

#[derive(Debug)]
pub enum ErrorKind {
    UnexpectedEof,
    UnterminatedString,
    UnexpectedCharacter,
}