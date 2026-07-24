// interp.rs — tree-walking evaluator (वेगः engine)
// Subset slice 3: integers, strings, booleans, nil; मानय/ध्रुव, assignment,
// arithmetic (exact integers; '/' errors on non-divisible until decimals land),
// comparisons, च/वा/न, यदि, यावत्, विरम/अनुवर्त, वद, वाक्यम्, दैर्घ्यम्, प्रकारः.

use std::collections::HashMap;

use crate::ast::{Expr, Stmt};

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Int(i64),
    Str(String),
    Bool(bool),
    Nil,
}

// control-flow signal threaded through statement execution
enum Flow {
    Normal,
    Break,
    Continue,
}

pub struct Interp {
    vars: HashMap<String, Value>,
    consts: std::collections::HashSet<String>,
}

type RResult<T> = Result<T, String>;

fn dev_num(mut x: i64) -> String {
    // render an integer with Devanagari digits
    let digits = ['०','१','२','३','४','५','६','७','८','९'];
    if x == 0 { return "०".to_string(); }
    let neg = x < 0;
    if neg { x = -x; }
    let mut s = String::new();
    while x > 0 {
        s.insert(0, digits[(x % 10) as usize]);
        x /= 10;
    }
    if neg { s.insert(0, '-'); }
    s
}

impl Interp {
    pub fn new() -> Self {
        Interp { vars: HashMap::new(), consts: std::collections::HashSet::new() }
    }

    pub fn run(&mut self, stmts: &[Stmt]) -> RResult<()> {
        match self.exec_block(stmts)? {
            Flow::Normal => Ok(()),
            _ => Err("'विरम'/'अनुवर्त' चक्रात् बहिः न शक्यम् / break/continue outside loop".into()),
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
                if self.consts.contains(name) {
                    return Err(format!(
                        "दोषः पङ्क्तौ {} — '{}' ध्रुवः / '{}' is a constant", line, name, name));
                }
                let v = self.eval(expr)?;
                self.vars.insert(name.clone(), v);
                if *is_const { self.consts.insert(name.clone()); }
                Ok(Flow::Normal)
            }
            Stmt::Assign { name, expr, line } => {
                if self.consts.contains(name) {
                    return Err(format!(
                        "दोषः पङ्क्तौ {} — '{}' ध्रुवः / '{}' is a constant", line, name, name));
                }
                if !self.vars.contains_key(name) {
                    return Err(format!(
                        "दोषः पङ्क्तौ {} — '{}' अघोषितम् / '{}' not declared", line, name, name));
                }
                let v = self.eval(expr)?;
                self.vars.insert(name.clone(), v);
                Ok(Flow::Normal)
            }
            Stmt::ExprStmt(e) => { self.eval(e)?; Ok(Flow::Normal) }
            Stmt::Break(_) => Ok(Flow::Break),
            Stmt::Continue(_) => Ok(Flow::Continue),
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
            _ => Err("अत्र सत्यासत्यम् अपेक्षितम् / condition must be a boolean".into()),
        }
    }

    fn eval(&mut self, e: &Expr) -> RResult<Value> {
        match e {
            Expr::Int(v) => Ok(Value::Int(*v)),
            Expr::Str(s) => Ok(Value::Str(s.clone())),
            Expr::Bool(b) => Ok(Value::Bool(*b)),
            Expr::Nil => Ok(Value::Nil),
            Expr::Var(name, line) => match self.vars.get(name) {
                Some(v) => Ok(v.clone()),
                None => Err(format!("दोषः पङ्क्तौ {} — अज्ञातं नाम '{}' / unknown name '{}'",
                                    line, name, name)),
            },
            Expr::Unary(op, sub, line) => {
                let v = self.eval(sub)?;
                match op.as_str() {
                    "-" => match v {
                        Value::Int(n) => Ok(Value::Int(-n)),
                        _ => Err(format!("दोषः पङ्क्तौ {} — सङ्ख्या अपेक्षिता", line)),
                    },
                    "न" => match v {
                        Value::Bool(b) => Ok(Value::Bool(!b)),
                        _ => Err(format!("दोषः पङ्क्तौ {} — 'न' सत्यासत्यम् अपेक्षते", line)),
                    },
                    _ => Err("आन्तरिकदोषः".into()),
                }
            }
            Expr::Binary(op, l, r, line) => {
                // short-circuit logic
                if op == "च" || op == "वा" {
                    let lv = match self.eval(l)? {
                        Value::Bool(b) => b,
                        _ => return Err(format!("दोषः पङ्क्तौ {} — 'च'/'वा' सत्यासत्यम् अपेक्षेते", line)),
                    };
                    if op == "च" && !lv { return Ok(Value::Bool(false)); }
                    if op == "वा" && lv { return Ok(Value::Bool(true)); }
                    let rv = match self.eval(r)? {
                        Value::Bool(b) => b,
                        _ => return Err(format!("दोषः पङ्क्तौ {} — 'च'/'वा' सत्यासत्यम् अपेक्षेते", line)),
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
                        (Value::Int(a), Value::Int(b)) => a.checked_add(*b)
                            .map(Value::Int)
                            .ok_or_else(|| format!(
                                "दोषः पङ्क्तौ {} — सङ्ख्या अतिविशाला / integer overflow", line)),
                        (Value::Str(a), Value::Str(b)) => Ok(Value::Str(format!("{}{}", a, b))),
                        _ => Err(format!(
                            "दोषः पङ्क्तौ {} — वाक्यं सङ्ख्या च न मिश्रणीये / cannot mix text and number", line)),
                    },
                    "-" | "*" | "%" | "/" => match (&lv, &rv) {
                        (Value::Int(a), Value::Int(b)) => self.int_arith(op, *a, *b, *line),
                        _ => Err(format!("दोषः पङ्क्तौ {} — सङ्ख्ये अपेक्षिते / expected numbers", line)),
                    },
                    _ => Err("आन्तरिकदोषः".into()),
                }
            }
            Expr::Call(name, args, line) => {
                let mut vals = Vec::new();
                for a in args { vals.push(self.eval(a)?); }
                self.call(name, vals, *line)
            }
        }
    }

    // All integer arithmetic is overflow-CHECKED. Rust would panic in debug and
    // silently wrap in release; the Python reference has arbitrary precision, so
    // wrapping would be a silent wrong answer — unacceptable. We raise a proper
    // bilingual error instead (bignum support arrives with the decimal slice).
    fn int_arith(&self, op: &str, a: i64, b: i64, line: usize) -> RResult<Value> {
        let overflow = || format!(
            "दोषः पङ्क्तौ {} — सङ्ख्या अतिविशाला (पूर्णाङ्क-सीमातिक्रमः) / integer overflow",
            line);
        match op {
            "-" => a.checked_sub(b).map(Value::Int).ok_or_else(overflow),
            "*" => a.checked_mul(b).map(Value::Int).ok_or_else(overflow),
            "%" => {
                if b == 0 {
                    return Err(format!("दोषः पङ्क्तौ {} — शून्येन भागो न शक्यः", line));
                }
                // Python semantics: floored remainder (-७ % ३ == २), whereas
                // Rust's '%' truncates (-1). Match the reference exactly.
                let r = a.checked_rem(b).ok_or_else(overflow)?;
                let r = if (r != 0) && ((r < 0) != (b < 0)) { r + b } else { r };
                Ok(Value::Int(r))
            }
            "/" => {
                if b == 0 {
                    return Err(format!("दोषः पङ्क्तौ {} — शून्येन भागो न शक्यः", line));
                }
                // exact decimals arrive in a later slice; evenly-divisible
                // integer division is exact and safe to answer now.
                if a % b == 0 {
                    a.checked_div(b).map(Value::Int).ok_or_else(overflow)
                } else {
                    Err(format!(
                        "दोषः पङ्क्तौ {} — दशमांश-विभागः अग्रिमे स्लाइसे / decimal '/' not yet in veg engine",
                        line))
                }
            }
            _ => Err("आन्तरिकदोषः".into()),
        }
    }

    fn compare(&self, op: &str, a: &Value, b: &Value, line: usize) -> RResult<Value> {
        let ord = match (a, b) {
            (Value::Int(x), Value::Int(y)) => x.partial_cmp(y),
            (Value::Str(x), Value::Str(y)) => x.partial_cmp(y),
            _ => return Err(format!("दोषः पङ्क्तौ {} — तुलना समानप्रकारयोः एव", line)),
        };
        use std::cmp::Ordering::*;
        let r = match (op, ord) {
            ("<", Some(Less)) => true,
            (">", Some(Greater)) => true,
            ("<=", Some(Less)) | ("<=", Some(Equal)) => true,
            (">=", Some(Greater)) | (">=", Some(Equal)) => true,
            _ => false,
        };
        Ok(Value::Bool(r))
    }

    fn display(&self, v: &Value) -> String {
        match v {
            Value::Int(n) => dev_num(*n),
            Value::Str(s) => s.clone(),
            Value::Bool(b) => if *b { "सत्यम्".into() } else { "असत्यम्".into() },
            Value::Nil => "शून्यम्".into(),
        }
    }

    fn call(&mut self, name: &str, vals: Vec<Value>, line: usize) -> RResult<Value> {
        match name {
            "वद" => {
                let parts: Vec<String> = vals.iter().map(|v| self.display(v)).collect();
                println!("{}", parts.join(" "));
                Ok(Value::Nil)
            }
            "वाक्यम्" => {
                if vals.len() != 1 {
                    return Err(format!("दोषः पङ्क्तौ {} — वाक्यम्() एकम् एव गृह्णाति", line));
                }
                Ok(Value::Str(self.display(&vals[0])))
            }
            "दैर्घ्यम्" => match vals.get(0) {
                Some(Value::Str(s)) => Ok(Value::Int(s.chars().count() as i64)),
                _ => Err(format!("दोषः पङ्क्तौ {} — दैर्घ्यम्() वाक्यम् गृह्णाति", line)),
            },
            "प्रकारः" => {
                let t = match vals.get(0) {
                    Some(Value::Int(_)) => "पूर्णाङ्कः",
                    Some(Value::Str(_)) => "वाक्यम्",
                    Some(Value::Bool(_)) => "सत्यासत्यम्",
                    Some(Value::Nil) => "शून्यम्",
                    None => "शून्यम्",
                };
                Ok(Value::Str(t.into()))
            }
            "सङ्ख्या" => match vals.get(0) {
                Some(Value::Str(s)) => {
                    let ascii: String = s.trim().chars().map(|c| match c {
                        '०'..='९' => (b'0' + (c as u32 - '०' as u32) as u8) as char,
                        other => other,
                    }).collect();
                    ascii.parse::<i64>()
                        .map(Value::Int)
                        .map_err(|_| format!("दोषः पङ्क्तौ {} — '{}' सङ्ख्या न", line, s))
                }
                _ => Err(format!("दोषः पङ्क्तौ {} — सङ्ख्या() वाक्यम् गृह्णाति", line)),
            },
            _ => Err(format!(
                "दोषः पङ्क्तौ {} — अज्ञातो विधिः '{}' / unknown function (user functions in a later slice)",
                line, name)),
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

    // evaluate an expression-ish program and capture the last वद by hand:
    fn eval_expr(src: &str) -> Value {
        let toks = lex(src).unwrap();
        let stmts = Parser::new(toks).program().unwrap();
        // execute all but rely on a trailing "मानय __ = <expr>।"
        let mut it = Interp::new();
        it.run(&stmts).unwrap();
        it.vars.get("प").cloned().unwrap()
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

    // Python-compatible floored modulo: -७ % ३ == २ (Rust's raw % gives -1)
    #[test]
    fn modulo_matches_python_semantics() {
        assert_eq!(eval_expr("मानय प = ०-७। प = प % ३।"), Value::Int(2));
        assert_eq!(eval_expr("मानय प = ७ % ३।"), Value::Int(1));
    }

    // overflow must error, never wrap silently
    #[test]
    fn arithmetic_overflow_errors() {
        let big = "मानय क = ९२२३३७२०३६८५४७७५८०७। क = क + १।";
        assert!(run_ok(big).is_err());
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
            Value::Int(25));   // 1+3+5+7+9
    }
}
