// ast.rs — संस्कृता syntax tree (वेगः engine)

#[derive(Debug, Clone)]
pub enum Expr {
    Int(i64),
    Str(String),
    Bool(bool),
    Nil,
    Var(String, usize),                         // name, line
    Unary(String, Box<Expr>, usize),            // op, operand
    Binary(String, Box<Expr>, Box<Expr>, usize),// op, left, right
    Call(String, Vec<Expr>, usize),             // builtin/function name, args
}

// `line` fields are carried for error reporting in later slices.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum Stmt {
    Let { name: String, expr: Expr, is_const: bool, line: usize },
    Assign { name: String, expr: Expr, line: usize },
    ExprStmt(Expr),
    If { branches: Vec<(Expr, Vec<Stmt>)>, else_body: Option<Vec<Stmt>>, line: usize },
    While { cond: Expr, body: Vec<Stmt>, line: usize },
    Break(usize),
    Continue(usize),
}
