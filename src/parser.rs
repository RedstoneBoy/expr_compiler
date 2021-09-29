use crate::{ast::*, token::{Token, TokenKind}};

pub struct Parser<'a> {
    tokens: Vec<Token<'a>>,
    index: usize,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: Vec<Token<'a>>) -> Self {
        Parser {
            tokens,
            index: 0,
        }
    }

    pub fn parse(mut self) -> Result<Expr<'a>, Error<'a>> {
        self.eat_compound()
            .map(Expr::Block)
    }

    fn eat_compound(&mut self) -> Result<BlockExpr<'a>, Error<'a>> {
        let mut exprs = Vec::new();
        let ret;

        loop {
            let expr = self.eat_expr()?;

            if self.cur_token_kind() == Some(&TokenKind::Semicolon) {
                self.eat_token()?;
                exprs.push(expr);
            } else {
                ret = expr;
                break;
            }
        }

        Ok(BlockExpr { exprs, ret: Box::new(ret) })
    }

    fn eat_expr(&mut self) -> Result<Expr<'a>, Error<'a>> {
        let mut expr = self.eat_expr2()?;

        loop {
            let cmp = match self.cur_token_kind() {
                Some(TokenKind::EqualEqual) => CmpOp::Equal,
                Some(TokenKind::BangEqual) => CmpOp::NotEqual,
                _ => break,
            };
            self.eat_token()?;
            let right = self.eat_expr2()?;
            expr = Expr::Compare(cmp, Box::new(expr), Box::new(right));
        }

        Ok(expr)
    }

    fn eat_expr2(&mut self) -> Result<Expr<'a>, Error<'a>> {
        let mut expr = self.eat_expr3()?;

        loop {
            let cmp = match self.cur_token_kind() {
                Some(TokenKind::Greater) => CmpOp::Greater,
                Some(TokenKind::GreaterEqual) => CmpOp::GreaterEqual,
                Some(TokenKind::Less) => CmpOp::Less,
                Some(TokenKind::LessEqual) => CmpOp::LessEqual,
                _ => break,
            };
            self.eat_token()?;
            let right = self.eat_expr3()?;
            expr = Expr::Compare(cmp, Box::new(expr), Box::new(right));
        }

        Ok(expr)
    }

    fn eat_expr3(&mut self) -> Result<Expr<'a>, Error<'a>> {
        let mut expr = self.eat_expr4()?;

        loop {
            let math = match self.cur_token_kind() {
                Some(TokenKind::Star) => MathOp::Mul,
                Some(TokenKind::Slash) => MathOp::Div,
                _ => break,
            };
            self.eat_token()?;
            let right = self.eat_expr4()?;
            expr = Expr::MathBin(math, Box::new(expr), Box::new(right));
        }

        Ok(expr)
    }

    fn eat_expr4(&mut self) -> Result<Expr<'a>, Error<'a>> {
        let mut expr = self.eat_expr5()?;

        loop {
            let math = match self.cur_token_kind() {
                Some(TokenKind::Plus) => MathOp::Add,
                Some(TokenKind::Minus) => MathOp::Sub,
                _ => break,
            };
            self.eat_token()?;
            let right = self.eat_expr5()?;
            expr = Expr::MathBin(math, Box::new(expr), Box::new(right));
        }

        Ok(expr)
    }

    fn eat_expr5(&mut self) -> Result<Expr<'a>, Error<'a>> {
        let token = self.eat_token()?;
        Ok(match &token.kind {
            TokenKind::Number(num) => Expr::Number(*num),
            TokenKind::True => Expr::Bool(true),
            TokenKind::False => Expr::Bool(false),
            TokenKind::Minus => Expr::Negate(Box::new(self.eat_expr5()?)),
            TokenKind::Bang => Expr::Not(Box::new(self.eat_expr5()?)),
            TokenKind::Ident(var) => {
                if self.cur_token_kind() == Some(&TokenKind::LParen) {
                    self.eat_token()?;
                    let mut args = vec![];
                    while self.cur_token_kind() != Some(&TokenKind::RParen) {
                        args.push(self.eat_expr()?);
                        if self.cur_token_kind() != Some(&TokenKind::RParen) {
                            self.eat_token_kind(TokenKind::Comma)?;
                        }
                    }
                    self.eat_token()?;
                    Expr::Call(var, args)
                } else {
                    Expr::Var(var)
                }
            }
            TokenKind::Dollar => {
                let uvar = self.eat_ident()?;
                loop {
                    let math_op = match self.cur_token_kind() {
                        Some(TokenKind::Equal) => {
                            self.eat_token()?;
                            let expr = self.eat_expr().map(Box::new)?;
                            break Expr::Assign(uvar, expr);
                        }
                        Some(TokenKind::PlusEqual) => MathOp::Add,
                        Some(TokenKind::MinusEqual) => MathOp::Sub,
                        Some(TokenKind::MultiplyEqual) => MathOp::Mul,
                        Some(TokenKind::DivideEqual) => MathOp::Div,
                        _ => break Expr::UserVar(uvar),
                    };

                    self.eat_token()?;
                    let expr = self.eat_expr().map(Box::new)?;
                    break Expr::MathAssign(math_op, uvar, expr);
                }
            }
            TokenKind::LBrace => {
                let block = self.eat_compound()?;
                let _ = self.eat_token_kind(TokenKind::RBrace)?;
                Expr::Block(block)
            }
            TokenKind::LParen => {
                let expr = self.eat_expr()?;
                self.eat_token_kind(TokenKind::RParen)?;
                expr
            }
            TokenKind::If => {
                self.eat_token_kind(TokenKind::LParen)?;
                let cond = self.eat_expr().map(Box::new)?;
                self.eat_token_kind(TokenKind::RParen)?;
                let yes = self.eat_expr().map(Box::new)?;
                self.eat_token_kind(TokenKind::Else)?;
                let no = self.eat_expr().map(Box::new)?;
                Expr::IfElse { cond, yes, no }
            }
            _ => {
                let line = token.line;
                let col = token.col;
                return Err(Error::new(ErrorKind::UnexpectedToken(token, vec![
                    TokenKind::Number(0.0),
                    TokenKind::True,
                    TokenKind::False,
                    TokenKind::Minus,
                    TokenKind::Bang,
                    TokenKind::Ident(""),
                    TokenKind::Dollar,
                    TokenKind::LBrace,
                ]), line, col));
            }
        })
    }
    
    fn eat_ident(&mut self) -> Result<&'a str, Error<'a>> {
        let token = self.eat_token()?;
        let line = token.line;
        let col = token.col;
        match &token.kind {
            TokenKind::Ident(var) => Ok(var),
            _ => Err(Error::new(ErrorKind::UnexpectedToken(token, vec![TokenKind::Ident("")]), line, col)),
        }
    }

    fn eat_token_kind(&mut self, kind: TokenKind<'a>) -> Result<Token<'a>, Error<'a>> {
        let token = self.eat_token()?;
        if token.kind == kind {
            Ok(token)
        } else {
            let line = token.line;
            let col = token.col;
            Err(Error::new(ErrorKind::UnexpectedToken(token, vec![kind]), line, col))
        }
    }

    fn eat_token(&mut self) -> Result<Token<'a>, Error<'a>> {
        match self.cur_token() {
            Some(token) => {
                let token = token.clone();
                self.index += 1;
                Ok(token)
            }
            None => Err(Error::new(ErrorKind::UnexpectedEof, 0, 0))
        }
    }

    fn cur_token(&self) -> Option<&Token<'a>> {
        self.tokens.get(self.index)
    }

    fn cur_token_kind(&self) -> Option<&TokenKind<'a>> {
        self.tokens.get(self.index).map(|tok| &tok.kind)
    }
}

#[derive(Debug)]
pub struct Error<'a> {
    pub kind: ErrorKind<'a>,
    pub line: u32,
    pub col: u32,
}

impl<'a> Error<'a> {
    pub fn new(kind: ErrorKind<'a>, line: u32, col: u32) -> Self {
        Error { kind, line, col }
    }
}

#[derive(Debug)]
pub enum ErrorKind<'a> {
    UnexpectedEof,
    UnexpectedToken(Token<'a>, Vec<TokenKind<'a>>),
}