// ast.rs — संस्कृता syntax tree (वेगः engine)

/// A function parameter: optional kāraka role label + name.
/// e.g. `कर्म सन्देशः` → Param { karaka: Some("कर्म"), name: "सन्देशः" }
#[derive(Debug, Clone)]
pub struct Param {
    pub karaka: Option<String>,
    pub name: String,
}

/// A call argument: optional kāraka label + value.
/// e.g. `कर्म: "नमस्ते"` → Arg { karaka: Some("कर्म"), value: … }
#[derive(Debug, Clone)]
pub struct Arg {
    pub karaka: Option<String>,
    pub value: Expr,
}

#[derive(Debug, Clone)]
pub enum Expr {
    Int(i64),
    Str(String),
    Bool(bool),
    Nil,
    Var(String, usize),                          // name, line
    Unary(String, Box<Expr>, usize),             // op, operand
    Binary(String, Box<Expr>, Box<Expr>, usize), // op, left, right
    Call(String, Vec<Arg>, usize),               // callee name, args
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
    Func { name: String, params: Vec<Param>, body: Vec<Stmt>, line: usize },
    Return { expr: Option<Expr>, line: usize },
    Break(usize),
    Continue(usize),
}
