// interp.rs — tree-walking evaluator (वेगः engine)
// Slices 1–5: EXACT numbers (arbitrary-precision integers + exact decimals),
// strings, booleans, nil; मानय/ध्रुव, assignment, arithmetic (floored %),
// comparisons, च/वा/न, यदि, यावत्, विरम/अनुवर्त, विधि/फलम् with recursion
// and kāraka arguments, builtins वद/वाक्यम्/दैर्घ्यम्/प्रकारः/सङ्ख्या.

use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use crate::ast::{Arg, Expr, Param, Stmt};
use crate::bigint::BigInt;
use crate::decimal::Decimal;

/// Fractional precision for inexact division — matches the Python reference's
/// default decimal context (28 significant digits).
const DIV_DIGITS: usize = 28;

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub params: Vec<Param>,
    pub body: Vec<Stmt>,
}

#[derive(Debug, Clone)]
pub enum Value {
    /// पूर्णाङ्कः — arbitrary-precision integer (no overflow, ever)
    Int(BigInt),
    /// दशमांशः — exact decimal (०.१ + ०.२ == ०.३)
    Dec(Decimal),
    Str(String),
    Bool(bool),
    Nil,
    Func(Rc<Function>),
}

impl Value {
    pub fn int(v: i64) -> Value {
        Value::Int(BigInt::from_i64(v))
    }

    /// Numeric view for arithmetic: integers promote to decimals when mixed.
    fn as_decimal(&self) -> Option<Decimal> {
        match self {
            Value::Int(b) => Some(Decimal::from_bigint(b.clone())),
            Value::Dec(d) => Some(d.clone()),
            _ => None,
        }
    }

    fn is_number(&self) -> bool {
        matches!(self, Value::Int(_) | Value::Dec(_))
    }
}

// Values compare by content; two functions are equal only if they are the same
// object (matching the reference, where functions are compared by identity).
impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            // numbers compare by VALUE across kinds: ५ == ५.० is सत्यम्
            (Value::Int(_), Value::Int(_))
            | (Value::Int(_), Value::Dec(_))
            | (Value::Dec(_), Value::Int(_))
            | (Value::Dec(_), Value::Dec(_)) => {
                match (self.as_decimal(), other.as_decimal()) {
                    (Some(a), Some(b)) => a.eq_value(&b),
                    _ => false,
                }
            }
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

/// Render an ASCII numeric string with Devanagari digits (matches the
/// reference's to_dev_digits, and never uses scientific notation).
fn dev_digits(ascii: &str) -> String {
    let d = ['०', '१', '२', '३', '४', '५', '६', '७', '८', '९'];
    ascii.chars()
        .map(|c| if c.is_ascii_digit() { d[(c as u8 - b'0') as usize] } else { c })
        .collect()
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
            Expr::Num(s) => {
                if s.contains('.') {
                    Decimal::parse(s).map(Value::Dec).ok_or_else(|| {
                        format!("दोषः — अशुद्धा सङ्ख्या '{}' / malformed number", s)
                    })
                } else {
                    Ok(Value::Int(BigInt::from_digits(s)))
                }
            }
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
                        Value::Int(n) => Ok(Value::Int(n.neg())),
                        Value::Dec(d) => Ok(Value::Dec(d.neg())),
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
                    "+" if matches!((&lv, &rv), (Value::Str(_), Value::Str(_))) => {
                        match (&lv, &rv) {
                            (Value::Str(a), Value::Str(b)) => Ok(Value::Str(format!("{}{}", a, b))),
                            _ => unreachable!(),
                        }
                    }
                    "+" | "-" | "*" | "%" | "/" => {
                        if !lv.is_number() || !rv.is_number() {
                            let mixed = matches!(&lv, Value::Str(_)) || matches!(&rv, Value::Str(_));
                            return Err(if mixed {
                                format!(
                                    "दोषः पङ्क्तौ {} — वाक्यं सङ्ख्या च न मिश्रणीये — 'वाक्यम्()' प्रयुज्यताम्\n\
                                     Error at line {} — cannot mix text and number", line, line)
                            } else {
                                format!(
                                    "दोषः पङ्क्तौ {} — सङ्ख्ये अपेक्षिते\n\
                                     Error at line {} — expected numbers", line, line)
                            });
                        }
                        self.num_arith(op, &lv, &rv, *line)
                    }
                    _ => Err("आन्तरिकदोषः / internal error".into()),
                }
            }
            Expr::Call(name, args, line) => self.call(name, args, *line),
        }
    }

    /// Exact numeric arithmetic. Integer op integer stays an arbitrary-precision
    /// integer; anything involving a decimal produces an exact decimal; '/'
    /// yields an exact result when it divides evenly (in either kind), else a
    /// decimal carrying DIV_DIGITS fractional digits — matching the reference.
    fn num_arith(&self, op: &str, lv: &Value, rv: &Value, line: usize) -> RResult<Value> {
        let div_zero = || format!(
            "दोषः पङ्क्तौ {} — शून्येन भागो न शक्यः\n\
             Error at line {} — division by zero", line, line);

        // both whole numbers → exact integer path (no overflow, ever)
        if let (Value::Int(a), Value::Int(b)) = (lv, rv) {
            return match op {
                "+" => Ok(Value::Int(a.add(b))),
                "-" => Ok(Value::Int(a.sub(b))),
                "*" => Ok(Value::Int(a.mul(b))),
                "%" => a.rem_floor(b).map(Value::Int).ok_or_else(div_zero),
                "/" => {
                    if b.is_zero() {
                        return Err(div_zero());
                    }
                    let (q, r) = a.divmod_trunc(b).ok_or_else(div_zero)?;
                    if r.is_zero() {
                        Ok(Value::Int(q))           // exact: १० / ५ → २
                    } else {
                        let d = Decimal::from_bigint(a.clone())
                            .div(&Decimal::from_bigint(b.clone()), DIV_DIGITS)
                            .ok_or_else(div_zero)?;
                        Ok(Value::Dec(d))           // १ / ४ → ०.२५
                    }
                }
                _ => Err("आन्तरिकदोषः / internal error".into()),
            };
        }

        // at least one decimal → exact decimal path
        let a = lv.as_decimal().ok_or_else(|| format!(
            "दोषः पङ्क्तौ {} — सङ्ख्ये अपेक्षिते\nError at line {} — expected numbers",
            line, line))?;
        let b = rv.as_decimal().ok_or_else(|| format!(
            "दोषः पङ्क्तौ {} — सङ्ख्ये अपेक्षिते\nError at line {} — expected numbers",
            line, line))?;
        let out = match op {
            "+" => a.add(&b),
            "-" => a.sub(&b),
            "*" => a.mul(&b),
            "/" => a.div(&b, DIV_DIGITS).ok_or_else(div_zero)?,
            "%" => {
                // decimal modulo: follow the reference's floored semantics
                if b.is_zero() {
                    return Err(div_zero());
                }
                let q = a.div(&b, 0).ok_or_else(div_zero)?;
                let floor_q = match q.to_bigint_if_integral() {
                    Some(i) => Decimal::from_bigint(i),
                    None => q,
                };
                a.sub(&floor_q.mul(&b))
            }
            _ => return Err("आन्तरिकदोषः / internal error".into()),
        };
        // an exact whole result stays a whole number (५.० + ५.० → १०)
        match out.to_bigint_if_integral() {
            Some(i) if out.is_integer() => Ok(Value::Int(i)),
            _ => Ok(Value::Dec(out)),
        }
    }

    fn compare(&self, op: &str, a: &Value, b: &Value, line: usize) -> RResult<Value> {
        let ord = if a.is_number() && b.is_number() {
            match (a.as_decimal(), b.as_decimal()) {
                (Some(x), Some(y)) => Some(x.cmp_to(&y)),
                _ => None,
            }
        } else {
            match (a, b) {
                (Value::Str(x), Value::Str(y)) => x.partial_cmp(y),
                _ => return Err(format!(
                    "दोषः पङ्क्तौ {} — तुलना समानप्रकारयोः एव\n\
                     Error at line {} — can only compare two numbers or two texts",
                    line, line)),
            }
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
            Value::Int(n) => dev_digits(&n.to_string_signed()),
            Value::Dec(d) => dev_digits(&d.to_plain_string()),
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
                Some(Value::Str(s)) if vals.len() == 1 =>
                    Ok(Value::int(s.chars().count() as i64)),
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
                    Value::Dec(_) => "दशमांशः",
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
                    let bad = || format!(
                        "दोषः पङ्क्तौ {} — '{}' सङ्ख्या न\n\
                         Error at line {} — '{}' is not a number", line, s, line, s);
                    let body = ascii.strip_prefix('-').unwrap_or(&ascii);
                    if body.is_empty() || !body.chars().all(|c| c.is_ascii_digit() || c == '.') {
                        return Err(bad());
                    }
                    if ascii.contains('.') {
                        Decimal::parse(&ascii).map(Value::Dec).ok_or_else(bad)
                    } else {
                        let neg = ascii.starts_with('-');
                        let mut b = BigInt::from_digits(body);
                        if neg {
                            b = b.neg();
                        }
                        Ok(Value::Int(b))
                    }
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

    /// Rendered form of `प` — the clearest way to assert on exact numbers.
    fn shown(src: &str) -> String {
        let toks = lex(src).unwrap();
        let stmts = Parser::new(toks).program().unwrap();
        let mut it = Interp::new();
        it.run(&stmts).unwrap();
        let v = it.scopes[0].vars.get("प").cloned().unwrap();
        it.display(&v)
    }

    #[test]
    fn arithmetic() {
        assert_eq!(eval_expr("मानय प = २ + ३ * ४।"), Value::int(14));
        assert_eq!(eval_expr("मानय प = (२ + ३) * ४।"), Value::int(20));
        assert_eq!(eval_expr("मानय प = १० % ३।"), Value::int(1));
    }

    // ---- slice 5: exactness, the language's core promise ----

    #[test]
    fn point_one_plus_point_two_is_point_three() {
        assert_eq!(shown("मानय प = ०.१ + ०.२।"), "०.३");
        assert_eq!(eval_expr("मानय प = ०.१ + ०.२ == ०.३।"), Value::Bool(true));
    }

    #[test]
    fn decimals_print_plainly() {
        assert_eq!(shown("मानय प = ३.१४१५९।"), "३.१४१५९");
        assert_eq!(shown("मानय प = ०.००१ * ०.००१।"), "०.०००००१");
        assert_eq!(shown("मानय प = ०-२.५।"), "-२.५");
    }

    #[test]
    fn money_math_is_exact() {
        // ₹450.50 + ₹320.25 + ₹599.00 — the व्ययगणकः example's core
        assert_eq!(shown("मानय प = ४५०.५० + ३२०.२५ + ५९९.००।"), "१३६९.७५");
    }

    #[test]
    fn integers_are_arbitrary_precision() {
        // 25! overflows i64 — must be exact here
        assert_eq!(
            shown("विधि फ(म) { यदि (म <= १) { फलम् १। } फलम् म * फ(म - १)। } \
                   मानय प = फ(२५)।"),
            dev_digits("15511210043330985984000000"));
        // and beyond i64 by literal, too
        assert_eq!(
            shown("मानय प = ९२२३३७२०३६८५४७७५८०७ + १।"),
            dev_digits("9223372036854775808"));
    }

    #[test]
    fn division_exact_or_decimal() {
        assert_eq!(shown("मानय प = १० / ५।"), "२");          // stays whole
        assert_eq!(shown("मानय प = १ / ४।"), "०.२५");         // exact decimal
        assert!(shown("मानय प = १ / ३।").starts_with("०.३३३३"));
    }

    #[test]
    fn mixed_int_decimal_arithmetic() {
        assert_eq!(shown("मानय प = २ + ०.५।"), "२.५");
        assert_eq!(shown("मानय प = ५.० + ५.०।"), "१०");      // exact whole result
        assert_eq!(eval_expr("मानय प = ५ == ५.०।"), Value::Bool(true));
    }

    #[test]
    fn decimal_comparison_and_type() {
        assert_eq!(eval_expr("मानय प = ०.३० == ०.३।"), Value::Bool(true));
        assert_eq!(eval_expr("मानय प = ०.१ < ०.२।"), Value::Bool(true));
        assert_eq!(shown("मानय प = प्रकारः(०.५)।"), "दशमांशः");
        assert_eq!(shown("मानय प = प्रकारः(५)।"), "पूर्णाङ्कः");
        assert_eq!(shown("मानय प = प्रकारः(१० / ५)।"), "पूर्णाङ्कः");
    }

    #[test]
    fn to_number_handles_decimals_and_bignums() {
        assert_eq!(shown("मानय प = सङ्ख्या(\"४.५\") + ०.५।"), "५");
        assert_eq!(shown("मानय प = सङ्ख्या(\"९९९९९९९९९९९९९९९९९९९९\") + १।"),
                   dev_digits("100000000000000000000"));
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
            Value::int(5050));
    }

    #[test]
    fn if_else() {
        assert_eq!(
            eval_expr("मानय प = ०। यदि (५ > ३) { प = १। } अन्यथा { प = २। }"),
            Value::int(1));
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
        assert_eq!(eval_expr("मानय प = ०-७। प = प % ३।"), Value::int(2));
        assert_eq!(eval_expr("मानय प = ७ % ३।"), Value::int(1));
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
            Value::int(25));
    }

    // ---- slice 4: functions ----

    #[test]
    fn simple_function() {
        assert_eq!(
            eval_expr("विधि योग(क, ख) { फलम् क + ख। } मानय प = योग(२, ३)।"),
            Value::int(5));
    }

    #[test]
    fn recursion_factorial() {
        assert_eq!(
            eval_expr("विधि फ(म) { यदि (म <= १) { फलम् १। } फलम् म * फ(म - १)। } \
                       मानय प = फ(५)।"),
            Value::int(120));
    }

    #[test]
    fn recursion_fibonacci() {
        assert_eq!(
            eval_expr("विधि फिब(म) { यदि (म <= १) { फलम् म। } \
                       फलम् फिब(म - १) + फिब(म - २)। } मानय प = फिब(१०)।"),
            Value::int(55));
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
            Value::int(11));
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
            Value::int(4));
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
                it.run(&stmts)?;
                // extract a plain i64 here — Value holds an Rc and is not Send
                match it.scopes[0].vars.get("प") {
                    Some(Value::Int(n)) => Ok(*n),
                    other => Err(format!("unexpected value: {:?}", other)),
                }
            })
            .unwrap();
        let n: Result<i64, String> = handle.join().expect("thread must not crash");
        assert_eq!(n.unwrap(), 200);
    }
}
