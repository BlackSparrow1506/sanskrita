// interp.rs — tree-walking evaluator (वेगः engine)
// Slices 1–4: integers, strings, booleans, nil; मानय/ध्रुव, assignment,
// arithmetic (overflow-checked, Python-compatible floored %), comparisons,
// च/वा/न, यदि, यावत्, विरम/अनुवर्त, विधि/फलम् with recursion and kāraka
// arguments, builtins वद/वाक्यम्/दैर्घ्यम्/प्रकारः/सङ्ख्या.

use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use crate::ast::{Arg, Expr, Param, Stmt};

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub params: Vec<Param>,
    pub body: Vec<Stmt>,
}

#[derive(Debug, Clone)]
pub enum Value {
    Int(i64),
    Str(String),
    Bool(bool),
    Nil,
    Func(Rc<Function>),
}

// Values compare by content; two functions are equal only if they are the same
// object (matching the reference, where functions are compared by identity).
impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => a == b,
            (Value::Str(a), Value::Str(b)) => a == b,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Nil, Value::Nil) => true,
            (Value::Func(a), Value::Func(b)) => Rc::ptr_eq(a, b),
            _ => false,
        }
    }
}

/// A lexical scope. `parent` indexes into Interp::scopes (arena), so recursion
/// and nesting stay cheap and borrow-checker friendly.
struct Scope {
    vars: HashMap<String, Value>,
    consts: HashSet<String>,
    parent: Option<usize>,
}

impl Scope {
    fn new(parent: Option<usize>) -> Self {
        Scope { vars: HashMap::new(), consts: HashSet::new(), parent }
    }
}

// control-flow signal threaded through statement execution
enum Flow {
    Normal,
    Break,
    Continue,
    Return(Value),
}

pub struct Interp {
    scopes: Vec<Scope>,
    current: usize,
    depth: usize,
    max_depth: usize,
    /// Address of a local in the frame where this interpreter was created.
    /// Stacks grow downward, so (anchor − current address) ≈ bytes of stack
    /// consumed. Measuring beats guessing frame sizes, which vary wildly
    /// between debug and release builds.
    anchor: usize,
    stack_budget: usize,
}

type RResult<T> = Result<T, String>;

/// A depth cap is a *secondary* guard; the primary one is the measured stack
/// budget below. This value is high because it should rarely be what fires.
pub const DEFAULT_MAX_DEPTH: usize = 100_000;

/// Default stack budget: 1 MB, safe inside the ~2 MB stack that `cargo test`
/// gives a spawned test thread, on debug builds where frames are largest.
/// The binary raises this — see main.rs (256 MB stack, 192 MB budget).
pub const DEFAULT_STACK_BUDGET: usize = 1024 * 1024;

fn dev_num(mut x: i64) -> String {
    let digits = ['०', '१', '२', '३', '४', '५', '६', '७', '८', '९'];
    if x == 0 {
        return "०".to_string();
    }
    let neg = x < 0;
    let mut s = String::new();
    // handle i64::MIN safely by working with the absolute value in i128
    let mut v: i128 = x as i128;
    if neg {
        v = -v;
    }
    x = 0;
    let _ = x;
    while v > 0 {
        s.insert(0, digits[(v % 10) as usize]);
        v /= 10;
    }
    if neg {
        s.insert(0, '-');
    }
    s
}

impl Interp {
    pub fn new() -> Self {
        let probe = 0u8;
        Interp {
            scopes: vec![Scope::new(None)],
            current: 0,
            depth: 0,
            max_depth: DEFAULT_MAX_DEPTH,
            anchor: &probe as *const u8 as usize,
            stack_budget: DEFAULT_STACK_BUDGET,
        }
    }

    /// Raise the limits — only safe when the caller has arranged a
    /// correspondingly large stack (see main.rs).
    pub fn with_limits(mut self, max_depth: usize, stack_budget: usize) -> Self {
        self.max_depth = max_depth;
        self.stack_budget = stack_budget;
        self
    }

    /// Approximate bytes of stack consumed since the interpreter was created.
    #[inline]
    fn stack_used(&self) -> usize {
        let probe = 0u8;
        let here = &probe as *const u8 as usize;
        self.anchor.saturating_sub(here)   // stacks grow downward
    }

    // ---- scope helpers ----

    fn lookup(&self, name: &str) -> Option<&Value> {
        let mut idx = Some(self.current);
        while let Some(i) = idx {
            if let Some(v) = self.scopes[i].vars.get(name) {
                return Some(v);
            }
            idx = self.scopes[i].parent;
        }
        None
    }

    /// Find the scope index that holds `name`, walking outward.
    fn scope_of(&self, name: &str) -> Option<usize> {
        let mut idx = Some(self.current);
        while let Some(i) = idx {
            if self.scopes[i].vars.contains_key(name) {
                return Some(i);
            }
            idx = self.scopes[i].parent;
        }
        None
    }

    pub fn run(&mut self, stmts: &[Stmt]) -> RResult<()> {
        match self.exec_block(stmts)? {
            Flow::Normal => Ok(()),
            Flow::Return(_) => Err(
                "'फलम्' विधेः बहिः न शक्यम् / 'फलम्' (return) only works inside a विधि".into()),
            _ => Err(
                "'विरम'/'अनुवर्त' चक्रात् बहिः न शक्यम् / break/continue outside a loop".into()),
        }
    }

    fn exec_block(&mut self, stmts: &[Stmt]) -> RResult<Flow> {
        for st in stmts {
            match self.exec(st)? {
                Flow::Normal => {}
                other => return Ok(other),
            }
        }
        Ok(Flow::Normal)
    }

    fn exec(&mut self, st: &Stmt) -> RResult<Flow> {
        match st {
            Stmt::Let { name, expr, is_const, line } => {
                if self.scopes[self.current].consts.contains(name) {
                    return Err(format!(
                        "दोषः पङ्क्तौ {} — '{}' ध्रुवः — परिवर्तनं न शक्यम्\n\
                         Error at line {} — '{}' is a constant", line, name, line, name));
                }
                let v = self.eval(expr)?;
                let cur = self.current;
                self.scopes[cur].vars.insert(name.clone(), v);
                if *is_const {
                    self.scopes[cur].consts.insert(name.clone());
                }
                Ok(Flow::Normal)
            }
            Stmt::Assign { name, expr, line } => {
                let v = self.eval(expr)?;
                match self.scope_of(name) {
                    Some(i) => {
                        if self.scopes[i].consts.contains(name) {
                            return Err(format!(
                                "दोषः पङ्क्तौ {} — '{}' ध्रुवः — परिवर्तनं न शक्यम्\n\
                                 Error at line {} — '{}' is a constant", line, name, line, name));
                        }
                        self.scopes[i].vars.insert(name.clone(), v);
                        Ok(Flow::Normal)
                    }
                    None => Err(format!(
                        "दोषः पङ्क्तौ {} — '{}' अघोषितम् — प्रथमं 'मानय' प्रयुज्यताम्\n\
                         Error at line {} — '{}' not declared", line, name, line, name)),
                }
            }
            Stmt::ExprStmt(e) => {
                self.eval(e)?;
                Ok(Flow::Normal)
            }
            Stmt::Break(_) => Ok(Flow::Break),
            Stmt::Continue(_) => Ok(Flow::Continue),
            Stmt::Return { expr, .. } => {
                let v = match expr {
                    Some(e) => self.eval(e)?,
                    None => Value::Nil,
                };
                Ok(Flow::Return(v))
            }
            Stmt::Func { name, params, body, .. } => {
                let f = Value::Func(Rc::new(Function {
                    name: name.clone(),
                    params: params.clone(),
                    body: body.clone(),
                }));
                let cur = self.current;
                self.scopes[cur].vars.insert(name.clone(), f);
                Ok(Flow::Normal)
            }
            Stmt::If { branches, else_body, .. } => {
                for (cond, body) in branches {
                    if self.truth(cond)? {
                        return self.exec_block(body);
                    }
                }
                if let Some(body) = else_body {
                    return self.exec_block(body);
                }
                Ok(Flow::Normal)
            }
            Stmt::While { cond, body, .. } => {
                while self.truth(cond)? {
                    match self.exec_block(body)? {
                        Flow::Break => break,
                        Flow::Return(v) => return Ok(Flow::Return(v)),
                        Flow::Continue | Flow::Normal => {}
                    }
                }
                Ok(Flow::Normal)
            }
        }
    }

    fn truth(&mut self, e: &Expr) -> RResult<bool> {
        match self.eval(e)? {
            Value::Bool(b) => Ok(b),
            _ => Err("अत्र सत्यासत्यम् अपेक्षितम् (सत्यम्/असत्यम्)\n\
                      Error — condition must be सत्यम्/असत्यम् (a boolean)".into()),
        }
    }

    fn eval(&mut self, e: &Expr) -> RResult<Value> {
        match e {
            Expr::Int(v) => Ok(Value::Int(*v)),
            Expr::Str(s) => Ok(Value::Str(s.clone())),
            Expr::Bool(b) => Ok(Value::Bool(*b)),
            Expr::Nil => Ok(Value::Nil),
            Expr::Var(name, line) => match self.lookup(name) {
                Some(v) => Ok(v.clone()),
                None => Err(format!(
                    "दोषः पङ्क्तौ {} — अज्ञातं नाम '{}'\nError at line {} — unknown name '{}'",
                    line, name, line, name)),
            },
            Expr::Unary(op, sub, line) => {
                let v = self.eval(sub)?;
                match op.as_str() {
                    "-" => match v {
                        Value::Int(n) => n.checked_neg().map(Value::Int).ok_or_else(|| {
                            format!("दोषः पङ्क्तौ {} — सङ्ख्या अतिविशाला / integer overflow", line)
                        }),
                        _ => Err(format!(
                            "दोषः पङ्क्तौ {} — सङ्ख्या अपेक्षिता\nError at line {} — expected a number",
                            line, line)),
                    },
                    "न" => match v {
                        Value::Bool(b) => Ok(Value::Bool(!b)),
                        _ => Err(format!(
                            "दोषः पङ्क्तौ {} — 'न' सत्यासत्यम् एव अपेक्षते\n\
                             Error at line {} — 'न' (not) needs a boolean", line, line)),
                    },
                    _ => Err("आन्तरिकदोषः / internal error".into()),
                }
            }
            Expr::Binary(op, l, r, line) => {
                if op == "च" || op == "वा" {
                    let lv = match self.eval(l)? {
                        Value::Bool(b) => b,
                        _ => return Err(format!(
                            "दोषः पङ्क्तौ {} — 'च'/'वा' सत्यासत्यम् अपेक्षेते\n\
                             Error at line {} — 'च'/'वा' need booleans", line, line)),
                    };
                    if op == "च" && !lv {
                        return Ok(Value::Bool(false));
                    }
                    if op == "वा" && lv {
                        return Ok(Value::Bool(true));
                    }
                    let rv = match self.eval(r)? {
                        Value::Bool(b) => b,
                        _ => return Err(format!(
                            "दोषः पङ्क्तौ {} — 'च'/'वा' सत्यासत्यम् अपेक्षेते\n\
                             Error at line {} — 'च'/'वा' need booleans", line, line)),
                    };
                    return Ok(Value::Bool(rv));
                }
                let lv = self.eval(l)?;
                let rv = self.eval(r)?;
                match op.as_str() {
                    "==" => Ok(Value::Bool(lv == rv)),
                    "!=" => Ok(Value::Bool(lv != rv)),
                    "<" | ">" | "<=" | ">=" => self.compare(op, &lv, &rv, *line),
                    "+" => match (&lv, &rv) {
                        (Value::Int(a), Value::Int(b)) => a.checked_add(*b).map(Value::Int)
                            .ok_or_else(|| format!(
                                "दोषः पङ्क्तौ {} — सङ्ख्या अतिविशाला / integer overflow", line)),
                        (Value::Str(a), Value::Str(b)) => Ok(Value::Str(format!("{}{}", a, b))),
                        _ => Err(format!(
                            "दोषः पङ्क्तौ {} — वाक्यं सङ्ख्या च न मिश्रणीये — 'वाक्यम्()' प्रयुज्यताम्\n\
                             Error at line {} — cannot mix text and number", line, line)),
                    },
                    "-" | "*" | "%" | "/" => match (&lv, &rv) {
                        (Value::Int(a), Value::Int(b)) => self.int_arith(op, *a, *b, *line),
                        _ => Err(format!(
                            "दोषः पङ्क्तौ {} — सङ्ख्ये अपेक्षिते\nError at line {} — expected numbers",
                            line, line)),
                    },
                    _ => Err("आन्तरिकदोषः / internal error".into()),
                }
            }
            Expr::Call(name, args, line) => self.call(name, args, *line),
        }
    }

    // All integer arithmetic is overflow-CHECKED (see AUDIT.md #2) and '%'
    // follows Python's floored semantics (#3).
    fn int_arith(&self, op: &str, a: i64, b: i64, line: usize) -> RResult<Value> {
        let overflow = || format!(
            "दोषः पङ्क्तौ {} — सङ्ख्या अतिविशाला (पूर्णाङ्क-सीमातिक्रमः)\n\
             Error at line {} — integer overflow", line, line);
        match op {
            "-" => a.checked_sub(b).map(Value::Int).ok_or_else(overflow),
            "*" => a.checked_mul(b).map(Value::Int).ok_or_else(overflow),
            "%" => {
                if b == 0 {
                    return Err(format!(
                        "दोषः पङ्क्तौ {} — शून्येन भागो न शक्यः\n\
                         Error at line {} — division by zero", line, line));
                }
                let r = a.checked_rem(b).ok_or_else(overflow)?;
                let r = if (r != 0) && ((r < 0) != (b < 0)) { r + b } else { r };
                Ok(Value::Int(r))
            }
            "/" => {
                if b == 0 {
                    return Err(format!(
                        "दोषः पङ्क्तौ {} — शून्येन भागो न शक्यः\n\
                         Error at line {} — division by zero", line, line));
                }
                if a % b == 0 {
                    a.checked_div(b).map(Value::Int).ok_or_else(overflow)
                } else {
                    Err(format!(
                        "दोषः पङ्क्तौ {} — दशमांश-विभागः अग्रिमे स्लाइसे\n\
                         Error at line {} — decimal '/' not yet in the veg engine",
                        line, line))
                }
            }
            _ => Err("आन्तरिकदोषः / internal error".into()),
        }
    }

    fn compare(&self, op: &str, a: &Value, b: &Value, line: usize) -> RResult<Value> {
        let ord = match (a, b) {
            (Value::Int(x), Value::Int(y)) => x.partial_cmp(y),
            (Value::Str(x), Value::Str(y)) => x.partial_cmp(y),
            _ => return Err(format!(
                "दोषः पङ्क्तौ {} — तुलना समानप्रकारयोः एव\n\
                 Error at line {} — can only compare two numbers or two texts", line, line)),
        };
        use std::cmp::Ordering::*;
        let r = matches!(
            (op, ord),
            ("<", Some(Less)) | (">", Some(Greater))
                | ("<=", Some(Less)) | ("<=", Some(Equal))
                | (">=", Some(Greater)) | (">=", Some(Equal))
        );
        Ok(Value::Bool(r))
    }

    fn display(&self, v: &Value) -> String {
        match v {
            Value::Int(n) => dev_num(*n),
            Value::Str(s) => s.clone(),
            Value::Bool(b) => if *b { "सत्यम्".into() } else { "असत्यम्".into() },
            Value::Nil => "शून्यम्".into(),
            Value::Func(f) => format!("<विधिः {}>", f.name),
        }
    }

    // ---- calls ----

    fn call(&mut self, name: &str, args: &[Arg], line: usize) -> RResult<Value> {
        // user-defined function?
        if let Some(Value::Func(f)) = self.lookup(name).cloned() {
            return self.call_function(&f, args, line);
        }
        // builtin — builtins take positional arguments only
        let mut vals = Vec::with_capacity(args.len());
        for a in args {
            if a.karaka.is_some() {
                return Err(format!(
                    "दोषः पङ्क्तौ {} — अन्तर्निहितविधयः कारकं न गृह्णन्ति\n\
                     Error at line {} — builtins do not take kāraka labels", line, line));
            }
            vals.push(self.eval(&a.value)?);
        }
        self.call_builtin(name, vals, line)
    }

    fn call_function(&mut self, f: &Rc<Function>, args: &[Arg], line: usize) -> RResult<Value> {
        // Primary guard: measured stack use. Secondary: call depth. Either
        // firing gives a clean bilingual error instead of a process crash.
        if self.stack_used() > self.stack_budget || self.depth >= self.max_depth {
            return Err(format!(
                "दोषः पङ्क्तौ {} — अतिगभीरा पुनरावृत्तिः (स्मृति-सीमा)\n\
                 Error at line {} — recursion too deep (stack limit)", line, line));
        }
        // evaluate arguments in the CALLER's scope
        let mut positional: Vec<Value> = Vec::new();
        let mut labeled: Vec<(String, Value)> = Vec::new();
        for a in args {
            let v = self.eval(&a.value)?;
            match &a.karaka {
                Some(k) => labeled.push((k.clone(), v)),
                None => positional.push(v),
            }
        }
        // bind parameters: kāraka labels first (any order), then positionally
        let mut local = Scope::new(Some(0)); // functions close over globals
        let mut pos_iter = positional.into_iter();
        for p in &f.params {
            let bound = if let Some(k) = &p.karaka {
                if let Some(idx) = labeled.iter().position(|(lk, _)| lk == k) {
                    Some(labeled.remove(idx).1)
                } else {
                    pos_iter.next()
                }
            } else {
                pos_iter.next()
            };
            match bound {
                Some(v) => { local.vars.insert(p.name.clone(), v); }
                None => {
                    let role = p.karaka.as_deref().map(|k| format!(" ({})", k)).unwrap_or_default();
                    return Err(format!(
                        "दोषः पङ्क्तौ {} — '{}' विधौ '{}'{} इत्यस्य मूल्यं न दत्तम्\n\
                         Error at line {} — function '{}' missing argument '{}'{}",
                        line, f.name, p.name, role, line, f.name, p.name, role));
                }
            }
        }
        if let Some((k, _)) = labeled.first() {
            let valid: Vec<&str> = f.params.iter()
                .filter_map(|p| p.karaka.as_deref()).collect();
            let valid = if valid.is_empty() { "—".to_string() } else { valid.join(", ") };
            return Err(format!(
                "दोषः पङ्क्तौ {} — '{}' विधौ अज्ञातं कारकम् '{}' — विधेः कारकाणि: {}\n\
                 Error at line {} — function '{}' has no role '{}' — its roles are: {}",
                line, f.name, k, valid, line, f.name, k, valid));
        }
        if pos_iter.next().is_some() {
            return Err(format!(
                "दोषः पङ्क्तौ {} — '{}' विधौ अधिकानि मूल्यानि दत्तानि\n\
                 Error at line {} — too many arguments for function '{}'",
                line, f.name, line, f.name));
        }

        // push scope, run body, pop
        self.scopes.push(local);
        let saved = self.current;
        self.current = self.scopes.len() - 1;
        self.depth += 1;
        let result = self.exec_block(&f.body);
        self.depth -= 1;
        self.current = saved;
        self.scopes.pop();

        match result? {
            Flow::Return(v) => Ok(v),
            Flow::Normal => Ok(Value::Nil),
            _ => Err(format!(
                "दोषः पङ्क्तौ {} — 'विरम'/'अनुवर्त' चक्रात् बहिः न शक्यम्\n\
                 Error at line {} — break/continue outside a loop", line, line)),
        }
    }

    fn call_builtin(&mut self, name: &str, vals: Vec<Value>, line: usize) -> RResult<Value> {
        match name {
            "वद" => {
                let parts: Vec<String> = vals.iter().map(|v| self.display(v)).collect();
                println!("{}", parts.join(" "));
                Ok(Value::Nil)
            }
            "वाक्यम्" => {
                if vals.len() != 1 {
                    return Err(format!(
                        "दोषः पङ्क्तौ {} — वाक्यम्() एकम् एव गृह्णाति\n\
                         Error at line {} — वाक्यम्() takes exactly one value", line, line));
                }
                Ok(Value::Str(self.display(&vals[0])))
            }
            "दैर्घ्यम्" => match vals.first() {
                Some(Value::Str(s)) if vals.len() == 1 => Ok(Value::Int(s.chars().count() as i64)),
                _ => Err(format!(
                    "दोषः पङ्क्तौ {} — दैर्घ्यम्() वाक्यम् एकं गृह्णाति\n\
                     Error at line {} — दैर्घ्यम्() takes one text value", line, line)),
            },
            "प्रकारः" => {
                if vals.len() != 1 {
                    return Err(format!(
                        "दोषः पङ्क्तौ {} — प्रकारः() एकम् एव गृह्णाति\n\
                         Error at line {} — प्रकारः() takes exactly one value", line, line));
                }
                let t = match &vals[0] {
                    Value::Int(_) => "पूर्णाङ्कः",
                    Value::Str(_) => "वाक्यम्",
                    Value::Bool(_) => "सत्यासत्यम्",
                    Value::Nil => "शून्यम्",
                    Value::Func(_) => "विधिः",
                };
                Ok(Value::Str(t.into()))
            }
            "सङ्ख्या" => match vals.first() {
                Some(Value::Str(s)) if vals.len() == 1 => {
                    let ascii: String = s.trim().chars().map(|c| match c {
                        '०'..='९' => char::from(b'0' + (c as u32 - '०' as u32) as u8),
                        other => other,
                    }).collect();
                    ascii.parse::<i64>().map(Value::Int).map_err(|_| format!(
                        "दोषः पङ्क्तौ {} — '{}' सङ्ख्या न\n\
                         Error at line {} — '{}' is not a number", line, s, line, s))
                }
                _ => Err(format!(
                    "दोषः पङ्क्तौ {} — सङ्ख्या() वाक्यम् एकं गृह्णाति\n\
                     Error at line {} — सङ्ख्या() takes one text value", line, line)),
            },
            _ => Err(format!(
                "दोषः पङ्क्तौ {} — अज्ञातो विधिः '{}'\n\
                 Error at line {} — unknown function '{}'", line, name, line, name)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::lex;
    use crate::parser::Parser;

    fn run_ok(src: &str) -> Result<(), String> {
        let toks = lex(src)?;
        let stmts = Parser::new(toks).program()?;
        Interp::new().run(&stmts)
    }

    fn eval_expr(src: &str) -> Value {
        let toks = lex(src).unwrap();
        let stmts = Parser::new(toks).program().unwrap();
        let mut it = Interp::new();
        it.run(&stmts).unwrap();
        it.scopes[0].vars.get("प").cloned().unwrap()
    }

    #[test]
    fn arithmetic() {
        assert_eq!(eval_expr("मानय प = २ + ३ * ४।"), Value::Int(14));
        assert_eq!(eval_expr("मानय प = (२ + ३) * ४।"), Value::Int(20));
        assert_eq!(eval_expr("मानय प = १० % ३।"), Value::Int(1));
    }

    #[test]
    fn logic_and_compare() {
        assert_eq!(eval_expr("मानय प = ५ > ३ च २ < ४।"), Value::Bool(true));
        assert_eq!(eval_expr("मानय प = न सत्यम्।"), Value::Bool(false));
    }

    #[test]
    fn loop_sum() {
        assert_eq!(
            eval_expr("मानय प = ०। मानय इ = १। यावत् (इ <= १००) { प = प + इ। इ = इ + १। }"),
            Value::Int(5050));
    }

    #[test]
    fn if_else() {
        assert_eq!(
            eval_expr("मानय प = ०। यदि (५ > ३) { प = १। } अन्यथा { प = २। }"),
            Value::Int(1));
    }

    #[test]
    fn strings() {
        assert_eq!(eval_expr("मानय प = \"अ\" + \"ब\"।"), Value::Str("अब".into()));
    }

    #[test]
    fn const_guard() {
        assert!(run_ok("ध्रुव क = ५। क = ६।").is_err());
    }

    #[test]
    fn modulo_matches_python_semantics() {
        assert_eq!(eval_expr("मानय प = ०-७। प = प % ३।"), Value::Int(2));
        assert_eq!(eval_expr("मानय प = ७ % ३।"), Value::Int(1));
    }

    #[test]
    fn arithmetic_overflow_errors() {
        assert!(run_ok("मानय क = ९२२३३७२०३६८५४७७५८०७। क = क + १।").is_err());
    }

    #[test]
    fn undeclared_assignment_errors() {
        assert!(run_ok("क = ५।").is_err());
    }

    #[test]
    fn non_boolean_condition_errors() {
        assert!(run_ok("यदि (५) { वद(\"अ\")। }").is_err());
    }

    #[test]
    fn break_and_continue() {
        assert_eq!(
            eval_expr("मानय प = ०। मानय इ = ०। यावत् (सत्यम्) { इ = इ + १। \
                       यदि (इ % २ == ०) { अनुवर्त। } प = प + इ। \
                       यदि (इ >= ९) { विरम। } }"),
            Value::Int(25));
    }

    // ---- slice 4: functions ----

    #[test]
    fn simple_function() {
        assert_eq!(
            eval_expr("विधि योग(क, ख) { फलम् क + ख। } मानय प = योग(२, ३)।"),
            Value::Int(5));
    }

    #[test]
    fn recursion_factorial() {
        assert_eq!(
            eval_expr("विधि फ(म) { यदि (म <= १) { फलम् १। } फलम् म * फ(म - १)। } \
                       मानय प = फ(५)।"),
            Value::Int(120));
    }

    #[test]
    fn recursion_fibonacci() {
        assert_eq!(
            eval_expr("विधि फिब(म) { यदि (म <= १) { फलम् म। } \
                       फलम् फिब(म - १) + फिब(म - २)। } मानय प = फिब(१०)।"),
            Value::Int(55));
    }

    #[test]
    fn karaka_arguments_any_order() {
        let a = eval_expr("विधि प्रे(कर्म क, सम्प्रदान ख) { फलम् क + ख। } \
                           मानय प = प्रे(कर्म: \"अ\", सम्प्रदान: \"ब\")।");
        let b = eval_expr("विधि प्रे(कर्म क, सम्प्रदान ख) { फलम् क + ख। } \
                           मानय प = प्रे(सम्प्रदान: \"ब\", कर्म: \"अ\")।");
        assert_eq!(a, Value::Str("अब".into()));
        assert_eq!(a, b);   // order must not matter
    }

    #[test]
    fn unknown_karaka_errors() {
        assert!(run_ok("विधि प्रे(कर्म क) { फलम् क। } वद(प्रे(करण: \"अ\"))।").is_err());
    }

    #[test]
    fn non_karaka_label_errors() {
        assert!(run_ok("विधि प्रे(कर्म क) { फलम् क। } वद(प्रे(गलत: \"अ\"))।").is_err());
    }

    #[test]
    fn wrong_arity_errors() {
        assert!(run_ok("विधि योग(क, ख) { फलम् क + ख। } वद(योग(१))।").is_err());
        assert!(run_ok("विधि योग(क, ख) { फलम् क + ख। } वद(योग(१, २, ३))।").is_err());
    }

    #[test]
    fn function_scope_is_isolated() {
        // a local inside the function must not leak out
        assert!(run_ok("विधि फ() { मानय अन्तः = ५। फलम् अन्तः। } वद(फ())। वद(अन्तः)।").is_err());
    }

    #[test]
    fn function_sees_globals() {
        assert_eq!(
            eval_expr("मानय ग = १०। विधि फ() { फलम् ग + १। } मानय प = फ()।"),
            Value::Int(11));
    }

    #[test]
    fn function_without_return_gives_nil() {
        assert_eq!(eval_expr("विधि फ() { मानय क = १। } मानय प = फ()।"), Value::Nil);
    }

    #[test]
    fn return_from_inside_loop() {
        assert_eq!(
            eval_expr("विधि फ() { मानय इ = ०। यावत् (सत्यम्) { इ = इ + १। \
                       यदि (इ == ४) { फलम् इ। } } } मानय प = फ()।"),
            Value::Int(4));
    }

    // Runaway recursion must produce a clean error, never a process crash.
    // Run it inside a thread with a KNOWN stack and a budget set well below,
    // so the test is deterministic on every build profile and platform.
    #[test]
    fn deep_recursion_errors_not_crashes() {
        let handle = std::thread::Builder::new()
            .stack_size(8 * 1024 * 1024)
            .spawn(|| {
                let src = "विधि फ(म) { फलम् फ(म + १)। } वद(फ(१))।";
                let toks = lex(src).unwrap();
                let stmts = Parser::new(toks).program().unwrap();
                Interp::new()
                    .with_limits(1_000_000, 2 * 1024 * 1024)   // 2 MB of 8 MB
                    .run(&stmts)
            })
            .unwrap();
        let result = handle.join().expect("interpreter thread must not crash");
        assert!(result.is_err(), "runaway recursion must error, not succeed");
    }

    // A legitimately deep (but bounded) recursion must still work.
    #[test]
    fn moderate_recursion_works() {
        let handle = std::thread::Builder::new()
            .stack_size(8 * 1024 * 1024)
            .spawn(|| {
                let src = "विधि गण(म) { यदि (म <= ०) { फलम् ०। } फलम् १ + गण(म - १)। } \
                           मानय प = गण(२००)।";
                let toks = lex(src).unwrap();
                let stmts = Parser::new(toks).program().unwrap();
                let mut it = Interp::new().with_limits(1_000_000, 4 * 1024 * 1024);
                it.run(&stmts).map(|_| it.scopes[0].vars.get("प").cloned())
            })
            .unwrap();
        let v = handle.join().unwrap().unwrap();
        assert_eq!(v, Some(Value::Int(200)));
    }
}
