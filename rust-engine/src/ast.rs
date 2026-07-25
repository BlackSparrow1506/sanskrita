// ast.rs — संस्कृता syntax tree (वेगः engine)

/// A function parameter: optional kāraka role label + name.
/// e.g. `कर्म सन्देशः` → Param { karaka: Some("कर्म"), name: "सन्देशः" }
#[derive(Debug, Clone)]
pub struct Param {
    pub karaka: Option<String>,
    pub name: String,
    /// §2b — default value. The EXPRESSION is stored, never a shared value, so
    /// it is evaluated fresh on every call: Python's mutable-default bug
    /// (`def f(x=[])`) cannot happen here.
    pub default: Option<Expr>,
}

/// A call argument: optional kāraka label + value.
/// e.g. `कर्म: "नमस्ते"` → Arg { karaka: Some("कर्म"), value: … }
#[derive(Debug, Clone)]
pub struct Arg {
    pub karaka: Option<String>,
    pub value: Expr,
}

// Every node carries its source line for error messages; a few are not read
// yet (list literals cannot currently fail at runtime), hence the allow.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum Expr {
    /// Numeric literal as typed (digits, optional '.') — converted to an exact
    /// big integer or decimal at evaluation time, never to a float.
    Num(String),
    Str(String),
    Bool(bool),
    Nil,
    Var(String, usize),                          // name, line
    Unary(String, Box<Expr>, usize),             // op, operand
    Binary(String, Box<Expr>, Box<Expr>, usize), // op, left, right
    /// Call of any expression: `f(…)`, `वस्तु.विधि(…)`, `कोशः[क](…)`
    Call(Box<Expr>, Vec<Arg>, usize),
    /// `[अ, ब, स]`
    List(Vec<Expr>, usize),
    /// `{"क": १, "ख": २}`
    Map(Vec<(Expr, Expr)>, usize),
    /// `सूची[अनुक्रमः]` or `कोशः[कुञ्जिका]`
    Index(Box<Expr>, Box<Expr>, usize),
    /// `वस्तु.गुणः`
    Attr(Box<Expr>, String, usize),
    /// `सृज वर्गः(…)`
    New(Box<Expr>, usize),
}

/// Assignment targets: a name, an index, or an attribute.
#[derive(Debug, Clone)]
pub enum Target {
    Var(String),
    Index(Expr, Expr),
    Attr(Expr, String),
}

/// A method inside a वर्गः.
#[derive(Debug, Clone)]
pub struct Method {
    pub name: String,
    pub params: Vec<Param>,
    pub body: Vec<Stmt>,
}

// `line` fields are carried for error reporting.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum Stmt {
    Let { name: String, expr: Expr, is_const: bool,
          ty: Option<String>, nullable: bool, line: usize },
    Assign { target: Target, expr: Expr, line: usize },
    ExprStmt(Expr),
    If { branches: Vec<(Expr, Vec<Stmt>)>, else_body: Option<Vec<Stmt>>, line: usize },
    While { cond: Expr, body: Vec<Stmt>, line: usize },
    /// `प्रत्येकम् नाम इति संग्रहः { … }`
    ForEach { var: String, iter: Expr, body: Vec<Stmt>, line: usize },
    Func { name: String, params: Vec<Param>, body: Vec<Stmt>, line: usize },
    /// `वर्गः नाम : मातृवर्गः { विधि … }`
    Class { name: String, parent: Option<String>, methods: Vec<Method>, line: usize },
    /// `प्रयत { … } दोषे (त्रुटिः) { … }`
    Try { body: Vec<Stmt>, err_name: String, catch: Vec<Stmt>, line: usize },
    /// `आनय "संस्कृतम्" इति सं।` — native module or the user's own .सं file
    Import { module: String, alias: String, line: usize },
    /// `क्षिप "सन्देशः"।` — raise an error the program can catch
    Throw { expr: Expr, line: usize },
    Return { expr: Option<Expr>, line: usize },
    Break(usize),
    Continue(usize),
}
