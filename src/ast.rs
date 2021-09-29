
#[derive(Copy, Clone, Debug)]
pub enum MathOp {
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Copy, Clone, Debug)]
pub enum CmpOp {
    Equal,
    NotEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
}

#[derive(Debug)]
pub struct BlockExpr<'a> {
    pub exprs: Vec<Expr<'a>>,
    pub ret: Box<Expr<'a>>,
}

#[derive(Debug)]
pub enum Expr<'a> {
    Number(f64),
    Bool(bool),
    Var(&'a str),
    UserVar(&'a str),
    Block(BlockExpr<'a>),
    IfElse {
        cond: Box<Expr<'a>>,
        yes: Box<Expr<'a>>,
        no: Box<Expr<'a>>,
    },
    Call(&'a str, Vec<Expr<'a>>),

    Assign(&'a str, Box<Expr<'a>>),
    MathAssign(MathOp, &'a str, Box<Expr<'a>>),

    Negate(Box<Expr<'a>>),
    Not(Box<Expr<'a>>),
    MathBin(MathOp, Box<Expr<'a>>, Box<Expr<'a>>),
    Compare(CmpOp, Box<Expr<'a>>, Box<Expr<'a>>),
}