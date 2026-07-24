// interp.rs — tree-walking evaluator (वेगः engine), slices 1–6.
//
// Exact numbers (arbitrary-precision integers + exact decimals), strings,
// booleans, nil; मानय/ध्रुव, assignment to names/indices/attributes; arithmetic
// with floored %, comparisons, च/वा/न; यदि, यावत्, प्रत्येकम्…इति,
// विरम/अनुवर्त; विधि/फलम् with recursion and kāraka arguments; सूची and कोशः;
// वर्गः with inheritance, सृज, अयम्; प्रयत/दोषे; builtins.

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use crate::ast::{Arg, Expr, Method, Stmt, Target};
use crate::bigint::BigInt;
use crate::decimal::Decimal;
use crate::value::{Class, Function, Instance, Key, MapData, Value};

/// Fractional precision for inexact division — matches the reference's
/// default decimal context (28 significant digits).
const DIV_DIGITS: usize = 28;

/// A lexical scope. `parent` indexes into Interp::scopes (arena).
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
    anchor: usize,
    stack_budget: usize,
}

type RResult<T> = Result<T, String>;

pub const DEFAULT_MAX_DEPTH: usize = 100_000;
pub const DEFAULT_STACK_BUDGET: usize = 1024 * 1024;

/// Render an ASCII numeric string with Devanagari digits.
pub fn dev_digits(ascii: &str) -> String {
    let d = ['०', '१', '२', '३', '४', '५', '६', '७', '८', '९'];
    ascii.chars()
        .map(|c| if c.is_ascii_digit() { d[(c as u8 - b'0') as usize] } else { c })
        .collect()
}

fn err2(line: usize, sa: &str, en: &str) -> String {
    format!("दोषः पङ्क्तौ {} — {}\nError at line {} — {}",
            dev_digits(&line.to_string()), sa, line, en)
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

    pub fn with_limits(mut self, max_depth: usize, stack_budget: usize) -> Self {
        self.max_depth = max_depth;
        self.stack_budget = stack_budget;
        self
    }

    #[inline]
    fn stack_used(&self) -> usize {
        let probe = 0u8;
        let here = &probe as *const u8 as usize;
        self.anchor.saturating_sub(here)
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
                "'फलम्' विधेः बहिः न शक्यम्\n\
                 Error — 'फलम्' (return) only works inside a विधि".into()),
            _ => Err(
                "'विरम'/'अनुवर्त' चक्रात् बहिः न शक्यम्\n\
                 Error — break/continue outside a loop".into()),
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
                    return Err(err2(*line,
                        &format!("'{}' ध्रुवः — परिवर्तनं न शक्यम्", name),
                        &format!("'{}' is a constant", name)));
                }
                let v = self.eval(expr)?;
                let cur = self.current;
                self.scopes[cur].vars.insert(name.clone(), v);
                if *is_const {
                    self.scopes[cur].consts.insert(name.clone());
                }
                Ok(Flow::Normal)
            }
            Stmt::Assign { target, expr, line } => {
                let v = self.eval(expr)?;
                match target {
                    Target::Var(name) => match self.scope_of(name) {
                        Some(i) => {
                            if self.scopes[i].consts.contains(name) {
                                return Err(err2(*line,
                                    &format!("'{}' ध्रुवः — परिवर्तनं न शक्यम्", name),
                                    &format!("'{}' is a constant", name)));
                            }
                            self.scopes[i].vars.insert(name.clone(), v);
                        }
                        None => return Err(err2(*line,
                            &format!("'{}' अघोषितम् — प्रथमं 'मानय' प्रयुज्यताम्", name),
                            &format!("'{}' not declared", name))),
                    },
                    Target::Index(obj, idx) => {
                        let o = self.eval(obj)?;
                        let i = self.eval(idx)?;
                        match &o {
                            Value::List(l) => {
                                let pos = self.list_index(l.borrow().len(), &i, *line)?;
                                l.borrow_mut()[pos - 1] = v;
                            }
                            Value::Map(m) => {
                                let k = i.as_key().ok_or_else(|| err2(*line,
                                    "कुञ्जिका वाक्यं पूर्णाङ्कः वा भवेत्",
                                    "map keys must be text or whole numbers"))?;
                                m.borrow_mut().insert(k, v);
                            }
                            _ => return Err(err2(*line,
                                &format!("{} स्थानाङ्कं न गृह्णाति", o.type_name()),
                                "cannot index-assign into this value")),
                        }
                    }
                    Target::Attr(obj, name) => {
                        let o = self.eval(obj)?;
                        match &o {
                            Value::Object(inst) => {
                                inst.fields.borrow_mut().insert(name.clone(), v);
                            }
                            _ => return Err(err2(*line,
                                &format!("{} गुणं न गृह्णाति", o.type_name()),
                                "cannot set an attribute on this value")),
                        }
                    }
                }
                Ok(Flow::Normal)
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
            Stmt::Class { name, parent, methods, line } => {
                let parent_class = match parent {
                    Some(p) => match self.lookup(p) {
                        Some(Value::Class(c)) => Some(c.clone()),
                        _ => return Err(err2(*line,
                            &format!("मातृवर्गः '{}' न प्राप्तः", p),
                            &format!("parent class '{}' not found", p))),
                    },
                    None => None,
                };
                let mut map: HashMap<String, Rc<Function>> = HashMap::new();
                for Method { name: mname, params, body } in methods {
                    map.insert(mname.clone(), Rc::new(Function {
                        name: mname.clone(),
                        params: params.clone(),
                        body: body.clone(),
                    }));
                }
                let class = Value::Class(Rc::new(Class {
                    name: name.clone(),
                    parent: parent_class,
                    methods: map,
                }));
                let cur = self.current;
                self.scopes[cur].vars.insert(name.clone(), class);
                Ok(Flow::Normal)
            }
            Stmt::Try { body, err_name, catch, line: _ } => {
                match self.exec_block(body) {
                    Ok(flow) => Ok(flow),
                    Err(msg) => {
                        // bind the Sanskrit half of the message, like the reference
                        let sa = msg.lines().next().unwrap_or(&msg).to_string();
                        let cur = self.current;
                        self.scopes[cur].vars.insert(err_name.clone(), Value::Str(sa));
                        self.exec_block(catch)
                    }
                }
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
            Stmt::ForEach { var, iter, body, line } => {
                let seq = self.eval(iter)?;
                let items: Vec<Value> = match &seq {
                    Value::List(l) => l.borrow().clone(),
                    Value::Str(s) => s.chars().map(|c| Value::Str(c.to_string())).collect(),
                    Value::Map(m) => {
                        let b = m.borrow();
                        b.order.iter().map(|k| match k {
                            Key::Str(s) => Value::Str(s.clone()),
                            Key::Int(d) => {
                                let neg = d.starts_with('-');
                                let mag = BigInt::from_digits(
                                    d.strip_prefix('-').unwrap_or(d));
                                Value::Int(if neg { mag.neg() } else { mag })
                            }
                        }).collect()
                    }
                    _ => return Err(err2(*line,
                        &format!("प्रत्येकम् सूचीं कोशं वाक्यं वा अपेक्षते — {} प्राप्तम्",
                                 seq.type_name()),
                        "प्रत्येकम् needs a list, map, or text")),
                };
                for item in items {
                    let cur = self.current;
                    self.scopes[cur].vars.insert(var.clone(), item);
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

    /// Validate a 1-based index against a length, returning the 1-based position.
    fn list_index(&self, len: usize, idx: &Value, line: usize) -> RResult<usize> {
        let b = match idx {
            Value::Int(b) => b,
            _ => return Err(err2(line, "स्थानाङ्कः पूर्णाङ्कः भवेत्",
                                 "index must be a whole number")),
        };
        let i = b.to_i64().unwrap_or(i64::MAX);
        if i < 1 || i as usize > len {
            return Err(err2(line,
                &format!("स्थानाङ्कः {} सीमाबहिः (१..{})",
                         dev_digits(&i.to_string()), dev_digits(&len.to_string())),
                &format!("index {} out of range (1..{}) — संस्कृता counts from १", i, len)));
        }
        Ok(i as usize)
    }

    fn eval(&mut self, e: &Expr) -> RResult<Value> {
        match e {
            Expr::Num(s) => {
                if s.contains('.') {
                    Decimal::parse(s).map(Value::Dec).ok_or_else(||
                        format!("अशुद्धा सङ्ख्या '{}' / malformed number", s))
                } else {
                    Ok(Value::Int(BigInt::from_digits(s)))
                }
            }
            Expr::Str(s) => Ok(Value::Str(s.clone())),
            Expr::Bool(b) => Ok(Value::Bool(*b)),
            Expr::Nil => Ok(Value::Nil),
            Expr::Var(name, line) => match self.lookup(name) {
                Some(v) => Ok(v.clone()),
                None => Err(err2(*line,
                    &format!("अज्ञातं नाम '{}'", name),
                    &format!("unknown name '{}'", name))),
            },
            Expr::List(items, _) => {
                let mut out = Vec::with_capacity(items.len());
                for it in items {
                    out.push(self.eval(it)?);
                }
                Ok(Value::list(out))
            }
            Expr::Map(pairs, line) => {
                let mut data = MapData::default();
                for (k, v) in pairs {
                    let kv = self.eval(k)?;
                    let key = kv.as_key().ok_or_else(|| err2(*line,
                        "कुञ्जिका वाक्यं पूर्णाङ्कः वा भवेत्",
                        "map keys must be text or whole numbers"))?;
                    let vv = self.eval(v)?;
                    data.insert(key, vv);
                }
                Ok(Value::Map(Rc::new(RefCell::new(data))))
            }
            Expr::Index(obj, idx, line) => {
                let o = self.eval(obj)?;
                let i = self.eval(idx)?;
                match &o {
                    Value::List(l) => {
                        let pos = self.list_index(l.borrow().len(), &i, *line)?;
                        Ok(l.borrow()[pos - 1].clone())
                    }
                    Value::Str(s) => {
                        let chars: Vec<char> = s.chars().collect();
                        let pos = self.list_index(chars.len(), &i, *line)?;
                        Ok(Value::Str(chars[pos - 1].to_string()))
                    }
                    Value::Map(m) => {
                        let k = i.as_key().ok_or_else(|| err2(*line,
                            "कुञ्जिका वाक्यं पूर्णाङ्कः वा भवेत्",
                            "map keys must be text or whole numbers"))?;
                        let shown = self.display(&i);
                        m.borrow().get(&k).cloned().ok_or_else(|| err2(*line,
                            &format!("कुञ्जिका '{}' कोशे नास्ति", shown),
                            &format!("key '{}' not found in the कोशः", shown)))
                    }
                    _ => Err(err2(*line,
                        &format!("{} स्थानाङ्कं न गृह्णाति", o.type_name()),
                        "cannot index into this value")),
                }
            }
            Expr::Attr(obj, name, line) => {
                let o = self.eval(obj)?;
                match &o {
                    Value::Object(inst) => {
                        if let Some(v) = inst.fields.borrow().get(name) {
                            return Ok(v.clone());
                        }
                        match inst.class.find_method(name) {
                            Some(m) => Ok(Value::Bound(inst.clone(), m)),
                            None => Err(err2(*line,
                                &format!("'{}' वस्तुनि '{}' नास्ति", inst.class.name, name),
                                &format!("'{}' object has no '{}'", inst.class.name, name))),
                        }
                    }
                    _ => Err(err2(*line,
                        &format!("{} '.{}' न जानाति", o.type_name(), name),
                        &format!("this value has no attribute '.{}'", name))),
                }
            }
            Expr::New(inner, line) => {
                // `सृज वर्गः(…)` — the inner expression is the class call
                match inner.as_ref() {
                    Expr::Call(callee, args, _) => {
                        let c = self.eval(callee)?;
                        match c {
                            Value::Class(class) => self.instantiate(&class, args, *line),
                            other => Err(err2(*line,
                                &format!("सृज-अनन्तरं वर्गः अपेक्षितः — {} प्राप्तम्",
                                         other.type_name()),
                                "सृज must be followed by a class call — सृज वर्गः(...)")),
                        }
                    }
                    _ => Err(err2(*line,
                        "सृज-अनन्तरं वर्गाह्वानम् अपेक्षितम्",
                        "सृज must be followed by a class call — सृज वर्गः(...)")),
                }
            }
            Expr::Unary(op, sub, line) => {
                let v = self.eval(sub)?;
                match op.as_str() {
                    "-" => match v {
                        Value::Int(n) => Ok(Value::Int(n.neg())),
                        Value::Dec(d) => Ok(Value::Dec(d.neg())),
                        _ => Err(err2(*line, "सङ्ख्या अपेक्षिता", "expected a number")),
                    },
                    "न" => match v {
                        Value::Bool(b) => Ok(Value::Bool(!b)),
                        _ => Err(err2(*line, "'न' सत्यासत्यम् एव अपेक्षते",
                                      "'न' (not) needs a boolean")),
                    },
                    _ => Err("आन्तरिकदोषः / internal error".into()),
                }
            }
            Expr::Binary(op, l, r, line) => {
                if op == "च" || op == "वा" {
                    let lv = match self.eval(l)? {
                        Value::Bool(b) => b,
                        _ => return Err(err2(*line, "'च'/'वा' सत्यासत्यम् अपेक्षेते",
                                             "'च'/'वा' need booleans")),
                    };
                    if op == "च" && !lv {
                        return Ok(Value::Bool(false));
                    }
                    if op == "वा" && lv {
                        return Ok(Value::Bool(true));
                    }
                    let rv = match self.eval(r)? {
                        Value::Bool(b) => b,
                        _ => return Err(err2(*line, "'च'/'वा' सत्यासत्यम् अपेक्षेते",
                                             "'च'/'वा' need booleans")),
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
                    "+" if matches!((&lv, &rv), (Value::List(_), Value::List(_))) => {
                        match (&lv, &rv) {
                            (Value::List(a), Value::List(b)) => {
                                let mut out = a.borrow().clone();
                                out.extend(b.borrow().iter().cloned());
                                Ok(Value::list(out))
                            }
                            _ => unreachable!(),
                        }
                    }
                    "+" | "-" | "*" | "%" | "/" => {
                        if !lv.is_number() || !rv.is_number() {
                            let mixed = matches!(&lv, Value::Str(_)) || matches!(&rv, Value::Str(_));
                            return Err(if mixed {
                                err2(*line,
                                    "वाक्यं सङ्ख्या च न मिश्रणीये — 'वाक्यम्()' प्रयुज्यताम्",
                                    "cannot mix text and number — convert with वाक्यम्()")
                            } else {
                                err2(*line, "सङ्ख्ये अपेक्षिते", "expected numbers")
                            });
                        }
                        self.num_arith(op, &lv, &rv, *line)
                    }
                    _ => Err("आन्तरिकदोषः / internal error".into()),
                }
            }
            Expr::Call(callee, args, line) => {
                // builtin? (a bare name that isn't a user binding)
                if let Expr::Var(name, _) = callee.as_ref() {
                    if self.lookup(name).is_none() {
                        let mut vals = Vec::with_capacity(args.len());
                        for a in args {
                            if a.karaka.is_some() {
                                return Err(err2(*line,
                                    "अन्तर्निहितविधयः कारकं न गृह्णन्ति",
                                    "builtins do not take kāraka labels"));
                            }
                            vals.push(self.eval(&a.value)?);
                        }
                        return self.call_builtin(name, vals, *line);
                    }
                }
                let f = self.eval(callee)?;
                self.call_value(&f, args, *line)
            }
        }
    }

    fn instantiate(&mut self, class: &Rc<Class>, args: &[Arg], line: usize) -> RResult<Value> {
        let inst = Rc::new(Instance {
            class: class.clone(),
            fields: RefCell::new(HashMap::new()),
        });
        match class.find_method("आरम्भ") {
            Some(ctor) => {
                self.call_function(&ctor, args, line, Some(inst.clone()))?;
            }
            None if !args.is_empty() => {
                return Err(err2(line,
                    &format!("'{}' वर्गे 'आरम्भ' विधिः नास्ति", class.name),
                    &format!("class '{}' has no 'आरम्भ' constructor but got arguments",
                             class.name)));
            }
            None => {}
        }
        Ok(Value::Object(inst))
    }

    fn call_value(&mut self, f: &Value, args: &[Arg], line: usize) -> RResult<Value> {
        match f {
            Value::Func(func) => self.call_function(func, args, line, None),
            Value::Bound(inst, func) => self.call_function(func, args, line, Some(inst.clone())),
            Value::Class(class) => self.instantiate(class, args, line),
            other => Err(err2(line,
                &format!("{} आह्वातुं न शक्यम्", other.type_name()),
                "this value is not callable")),
        }
    }

    fn call_function(&mut self, f: &Rc<Function>, args: &[Arg], line: usize,
                     self_obj: Option<Rc<Instance>>) -> RResult<Value> {
        if self.stack_used() > self.stack_budget || self.depth >= self.max_depth {
            return Err(err2(line, "अतिगभीरा पुनरावृत्तिः (स्मृति-सीमा)",
                            "recursion too deep (stack limit)"));
        }
        let mut positional: Vec<Value> = Vec::new();
        let mut labeled: Vec<(String, Value)> = Vec::new();
        for a in args {
            let v = self.eval(&a.value)?;
            match &a.karaka {
                Some(k) => labeled.push((k.clone(), v)),
                None => positional.push(v),
            }
        }
        let mut local = Scope::new(Some(0));
        if let Some(obj) = &self_obj {
            local.vars.insert("अयम्".into(), Value::Object(obj.clone()));
        }
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
                    let role = p.karaka.as_deref().map(|k| format!(" ({})", k))
                        .unwrap_or_default();
                    return Err(err2(line,
                        &format!("'{}' विधौ '{}'{} इत्यस्य मूल्यं न दत्तम्", f.name, p.name, role),
                        &format!("function '{}' missing argument '{}'{}", f.name, p.name, role)));
                }
            }
        }
        if let Some((k, _)) = labeled.first() {
            let valid: Vec<&str> = f.params.iter().filter_map(|p| p.karaka.as_deref()).collect();
            let valid = if valid.is_empty() { "—".to_string() } else { valid.join(", ") };
            return Err(err2(line,
                &format!("'{}' विधौ अज्ञातं कारकम् '{}' — विधेः कारकाणि: {}", f.name, k, valid),
                &format!("function '{}' has no role '{}' — its roles are: {}", f.name, k, valid)));
        }
        if pos_iter.next().is_some() {
            return Err(err2(line,
                &format!("'{}' विधौ अधिकानि मूल्यानि दत्तानि", f.name),
                &format!("too many arguments for function '{}'", f.name)));
        }

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
            _ => Err(err2(line, "'विरम'/'अनुवर्त' चक्रात् बहिः न शक्यम्",
                          "break/continue outside a loop")),
        }
    }

    /// Exact numeric arithmetic (see slice 5).
    fn num_arith(&self, op: &str, lv: &Value, rv: &Value, line: usize) -> RResult<Value> {
        let div_zero = || err2(line, "शून्येन भागो न शक्यः", "division by zero");
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
                        Ok(Value::Dec(Decimal::from_bigint(q)))
                    } else {
                        let d = Decimal::from_bigint(a.clone())
                            .div(&Decimal::from_bigint(b.clone()), DIV_DIGITS)
                            .ok_or_else(div_zero)?;
                        Ok(Value::Dec(d))
                    }
                }
                _ => Err("आन्तरिकदोषः / internal error".into()),
            };
        }
        let a = lv.as_decimal().ok_or_else(|| err2(line, "सङ्ख्ये अपेक्षिते",
                                                   "expected numbers"))?;
        let b = rv.as_decimal().ok_or_else(|| err2(line, "सङ्ख्ये अपेक्षिते",
                                                   "expected numbers"))?;
        let out = match op {
            "+" => a.add(&b),
            "-" => a.sub(&b),
            "*" => a.mul(&b),
            "/" => a.div(&b, DIV_DIGITS).ok_or_else(div_zero)?,
            "%" => {
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
        Ok(Value::Dec(out))
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
                _ => return Err(err2(line, "तुलना समानप्रकारयोः एव",
                                     "can only compare two numbers or two texts")),
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

    pub fn display(&self, v: &Value) -> String {
        self.show(v, false)
    }

    fn show(&self, v: &Value, inner: bool) -> String {
        match v {
            Value::Int(n) => dev_digits(&n.to_string_signed()),
            Value::Dec(d) => dev_digits(&d.to_plain_string()),
            Value::Str(s) => if inner { format!("\"{}\"", s) } else { s.clone() },
            Value::Bool(b) => if *b { "सत्यम्".into() } else { "असत्यम्".into() },
            Value::Nil => "शून्यम्".into(),
            Value::Func(f) => format!("<विधिः {}>", f.name),
            Value::Bound(_, f) => format!("<विधिः {}>", f.name),
            Value::Class(c) => format!("<वर्गः {}>", c.name),
            Value::Object(o) => format!("<{} वस्तु>", o.class.name),
            Value::List(l) => {
                let parts: Vec<String> = l.borrow().iter().map(|x| self.show(x, true)).collect();
                format!("[{}]", parts.join(", "))
            }
            Value::Map(m) => {
                let b = m.borrow();
                let parts: Vec<String> = b.order.iter().map(|k| {
                    let kv = match k {
                        Key::Str(s) => format!("\"{}\"", s),
                        Key::Int(d) => dev_digits(d),
                    };
                    format!("{}: {}", kv, b.get(k).map(|x| self.show(x, true))
                        .unwrap_or_default())
                }).collect();
                format!("{{{}}}", parts.join(", "))
            }
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
                    return Err(err2(line, "वाक्यम्() एकम् एव गृह्णाति",
                                    "वाक्यम्() takes exactly one value"));
                }
                Ok(Value::Str(self.display(&vals[0])))
            }
            "दैर्घ्यम्" => match vals.first() {
                Some(Value::Str(s)) if vals.len() == 1 =>
                    Ok(Value::int(s.chars().count() as i64)),
                Some(Value::List(l)) if vals.len() == 1 =>
                    Ok(Value::int(l.borrow().len() as i64)),
                Some(Value::Map(m)) if vals.len() == 1 =>
                    Ok(Value::int(m.borrow().len() as i64)),
                _ => Err(err2(line, "दैर्घ्यम्() वाक्यं सूचीं कोशं वा गृह्णाति",
                              "दैर्घ्यम्() takes one text, list, or map")),
            },
            "प्रकारः" => {
                if vals.len() != 1 {
                    return Err(err2(line, "प्रकारः() एकम् एव गृह्णाति",
                                    "प्रकारः() takes exactly one value"));
                }
                Ok(Value::Str(vals[0].type_name().into()))
            }
            "सङ्ख्या" => match vals.first() {
                Some(Value::Str(s)) if vals.len() == 1 => {
                    let ascii: String = s.trim().chars().map(|c| match c {
                        '०'..='९' => char::from(b'0' + (c as u32 - '०' as u32) as u8),
                        other => other,
                    }).collect();
                    let bad = || err2(line, &format!("'{}' सङ्ख्या न", s),
                                      &format!("'{}' is not a number", s));
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
                _ => Err(err2(line, "सङ्ख्या() वाक्यम् एकं गृह्णाति",
                              "सङ्ख्या() takes one text value")),
            },
            "योजय" => match (vals.first(), vals.get(1)) {
                (Some(Value::List(l)), Some(v)) if vals.len() == 2 => {
                    l.borrow_mut().push(v.clone());
                    Ok(Value::Nil)
                }
                _ => Err(err2(line, "योजय(सूची, मूल्यम्) — द्वे अपेक्षिते",
                              "योजय(list, value) needs a list and a value")),
            },
            "अपनय" => match (vals.first(), vals.get(1)) {
                (Some(Value::List(l)), Some(idx)) if vals.len() == 2 => {
                    let pos = self.list_index(l.borrow().len(), idx, line)?;
                    Ok(l.borrow_mut().remove(pos - 1))
                }
                (Some(Value::Map(m)), Some(k)) if vals.len() == 2 => {
                    let key = k.as_key().ok_or_else(|| err2(line,
                        "कुञ्जिका वाक्यं पूर्णाङ्कः वा भवेत्",
                        "map keys must be text or whole numbers"))?;
                    let shown = self.display(k);
                    m.borrow_mut().remove(&key).ok_or_else(|| err2(line,
                        &format!("कुञ्जिका '{}' कोशे नास्ति", shown),
                        &format!("key '{}' not found in the कोशः", shown)))
                }
                _ => Err(err2(line, "अपनय(सूची, स्थानाङ्कः) / अपनय(कोशः, कुञ्जिका)",
                              "अपनय(list, index) or अपनय(map, key)")),
            },
            "कुञ्जिकाः" => match vals.first() {
                Some(Value::Map(m)) if vals.len() == 1 => {
                    let b = m.borrow();
                    let keys: Vec<Value> = b.order.iter().map(|k| match k {
                        Key::Str(s) => Value::Str(s.clone()),
                        Key::Int(d) => {
                            let neg = d.starts_with('-');
                            let mag = BigInt::from_digits(d.strip_prefix('-').unwrap_or(d));
                            Value::Int(if neg { mag.neg() } else { mag })
                        }
                    }).collect();
                    Ok(Value::list(keys))
                }
                _ => Err(err2(line, "कुञ्जिकाः(कोशः) — कोशः अपेक्षितः",
                              "कुञ्जिकाः(map) needs a map")),
            },
            "क्रमय" => match vals.first() {
                Some(Value::List(l)) if vals.len() == 1 => {
                    let items = l.borrow().clone();
                    let all_num = items.iter().all(|v| v.is_number());
                    let all_str = items.iter().all(|v| matches!(v, Value::Str(_)));
                    if !(all_num || all_str) {
                        return Err(err2(line, "मिश्रप्रकाराः क्रमयितुं न शक्याः",
                                        "cannot sort a list of mixed types"));
                    }
                    let mut out = items;
                    if all_num {
                        out.sort_by(|a, b| match (a.as_decimal(), b.as_decimal()) {
                            (Some(x), Some(y)) => x.cmp_to(&y),
                            _ => std::cmp::Ordering::Equal,
                        });
                    } else {
                        out.sort_by(|a, b| match (a, b) {
                            (Value::Str(x), Value::Str(y)) => x.cmp(y),
                            _ => std::cmp::Ordering::Equal,
                        });
                    }
                    Ok(Value::list(out))
                }
                _ => Err(err2(line, "क्रमय(सूची) — सूची अपेक्षिता",
                              "क्रमय(list) needs a list")),
            },
            "परिधिः" => match (vals.first(), vals.get(1)) {
                (Some(Value::Int(a)), Some(Value::Int(b))) if vals.len() == 2 => {
                    let (lo, hi) = (a.to_i64(), b.to_i64());
                    match (lo, hi) {
                        (Some(lo), Some(hi)) => {
                            let mut out = Vec::new();
                            let mut i = lo;
                            while i <= hi {
                                out.push(Value::int(i));
                                i += 1;
                            }
                            Ok(Value::list(out))
                        }
                        _ => Err(err2(line, "परिधिः अतिविशाला", "range too large")),
                    }
                }
                _ => Err(err2(line, "परिधिः(आदिः, अन्तः) — द्वौ पूर्णाङ्कौ अपेक्षितौ",
                              "परिधिः(start, end) needs two whole numbers")),
            },
            "पृच्छ" => {
                let prompt = vals.first().map(|v| self.display(v)).unwrap_or_default();
                use std::io::Write;
                print!("{}", prompt);
                let _ = std::io::stdout().flush();
                let mut line_in = String::new();
                match std::io::stdin().read_line(&mut line_in) {
                    Ok(_) => Ok(Value::Str(line_in.trim_end_matches(['\n', '\r']).to_string())),
                    Err(_) => Ok(Value::Str(String::new())),
                }
            }
            _ => Err(err2(line,
                &format!("अज्ञातो विधिः '{}'", name),
                &format!("unknown function '{}'", name))),
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

    fn shown(src: &str) -> String {
        let toks = lex(src).unwrap();
        let stmts = Parser::new(toks).program().unwrap();
        let mut it = Interp::new();
        it.run(&stmts).unwrap();
        let v = it.scopes[0].vars.get("प").cloned().unwrap();
        it.display(&v)
    }

    fn eval_expr(src: &str) -> Value {
        let toks = lex(src).unwrap();
        let stmts = Parser::new(toks).program().unwrap();
        let mut it = Interp::new();
        it.run(&stmts).unwrap();
        it.scopes[0].vars.get("प").cloned().unwrap()
    }

    // ---- slices 1–4 ----

    #[test]
    fn arithmetic() {
        assert_eq!(shown("मानय प = २ + ३ * ४।"), "१४");
        assert_eq!(shown("मानय प = (२ + ३) * ४।"), "२०");
        assert_eq!(shown("मानय प = १० % ३।"), "१");
    }

    #[test]
    fn logic_and_compare() {
        assert_eq!(eval_expr("मानय प = ५ > ३ च २ < ४।"), Value::Bool(true));
        assert_eq!(eval_expr("मानय प = न सत्यम्।"), Value::Bool(false));
    }

    #[test]
    fn loop_sum() {
        assert_eq!(
            shown("मानय प = ०। मानय इ = १। यावत् (इ <= १००) { प = प + इ। इ = इ + १। }"),
            "५०५०");
    }

    #[test]
    fn if_else() {
        assert_eq!(shown("मानय प = ०। यदि (५ > ३) { प = १। } अन्यथा { प = २। }"), "१");
    }

    #[test]
    fn strings() {
        assert_eq!(shown("मानय प = \"अ\" + \"ब\"।"), "अब");
    }

    #[test]
    fn const_guard() {
        assert!(run_ok("ध्रुव क = ५। क = ६।").is_err());
    }

    #[test]
    fn modulo_matches_python_semantics() {
        assert_eq!(shown("मानय प = ०-७। प = प % ३।"), "२");
        assert_eq!(shown("मानय प = ७ % ३।"), "१");
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
            shown("मानय प = ०। मानय इ = ०। यावत् (सत्यम्) { इ = इ + १। \
                   यदि (इ % २ == ०) { अनुवर्त। } प = प + इ। \
                   यदि (इ >= ९) { विरम। } }"),
            "२५");
    }

    #[test]
    fn simple_function_and_recursion() {
        assert_eq!(shown("विधि योग(क, ख) { फलम् क + ख। } मानय प = योग(२, ३)।"), "५");
        assert_eq!(
            shown("विधि फ(म) { यदि (म <= १) { फलम् १। } फलम् म * फ(म - १)। } मानय प = फ(५)।"),
            "१२०");
    }

    #[test]
    fn karaka_arguments_any_order() {
        let a = shown("विधि प्रे(कर्म क, सम्प्रदान ख) { फलम् क + ख। } \
                       मानय प = प्रे(कर्म: \"अ\", सम्प्रदान: \"ब\")।");
        let b = shown("विधि प्रे(कर्म क, सम्प्रदान ख) { फलम् क + ख। } \
                       मानय प = प्रे(सम्प्रदान: \"ब\", कर्म: \"अ\")।");
        assert_eq!(a, "अब");
        assert_eq!(a, b);
    }

    // ---- slice 5: exactness ----

    #[test]
    fn point_one_plus_point_two_is_point_three() {
        assert_eq!(shown("मानय प = ०.१ + ०.२।"), "०.३");
        assert_eq!(eval_expr("मानय प = ०.१ + ०.२ == ०.३।"), Value::Bool(true));
    }

    #[test]
    fn money_and_bignums() {
        assert_eq!(shown("मानय प = ४५०.५० + ३२०.२५ + ५९९.००।"), "१३६९.७५");
        assert_eq!(
            shown("विधि फ(म) { यदि (म <= १) { फलम् १। } फलम् म * फ(म - १)। } मानय प = फ(२५)।"),
            dev_digits("15511210043330985984000000"));
    }

    // ---- slice 6: collections ----

    #[test]
    fn list_literal_and_indexing() {
        assert_eq!(shown("मानय स = [१०, २०, ३०]। मानय प = स[१]।"), "१०");
        assert_eq!(shown("मानय स = [१०, २०, ३०]। मानय प = स[३]।"), "३०");
        assert_eq!(shown("मानय प = [१, २, ३]।"), "[१, २, ३]");
        assert_eq!(shown("मानय प = [\"अ\", \"ब\"]।"), "[\"अ\", \"ब\"]");
    }

    #[test]
    fn list_is_one_based_and_bounds_checked() {
        assert!(run_ok("मानय स = [१]। वद(स[०])।").is_err());
        assert!(run_ok("मानय स = [१]। वद(स[२])।").is_err());
    }

    #[test]
    fn list_mutation_and_builtins() {
        assert_eq!(shown("मानय स = [१, २]। योजय(स, ३)। मानय प = दैर्घ्यम्(स)।"), "३");
        assert_eq!(shown("मानय स = [१, २]। स[२] = ९९। मानय प = स[२]।"), "९९");
        assert_eq!(shown("मानय प = क्रमय([३, १, २])।"), "[१, २, ३]");
        assert_eq!(shown("मानय स = [१, २, ३]। मानय प = अपनय(स, १)।"), "१");
        assert_eq!(shown("मानय प = [१, २] + [३]।"), "[१, २, ३]");
    }

    #[test]
    fn lists_are_shared_references() {
        // assigning a list shares it, as in the reference
        assert_eq!(shown("मानय अ = [१]। मानय ब = अ। योजय(ब, २)। मानय प = दैर्घ्यम्(अ)।"), "२");
    }

    #[test]
    fn map_literal_index_and_keys() {
        assert_eq!(shown("मानय क = {\"नाम\": \"गौरी\"}। मानय प = क[\"नाम\"]।"), "गौरी");
        assert_eq!(shown("मानय क = {\"अ\": १}। क[\"ब\"] = २। मानय प = दैर्घ्यम्(क)।"), "२");
        assert_eq!(shown("मानय क = {\"अ\": १, \"ब\": २}। मानय प = कुञ्जिकाः(क)।"),
                   "[\"अ\", \"ब\"]");
        assert_eq!(shown("मानय क = {\"अ\": १, \"ब\": २}। अपनय(क, \"अ\")। \
                          मानय प = दैर्घ्यम्(क)।"), "१");
        assert!(run_ok("मानय क = {\"अ\": १}। वद(क[\"ख\"])।").is_err());
    }

    #[test]
    fn foreach_over_list_map_and_text() {
        assert_eq!(
            shown("मानय प = ०। प्रत्येकम् इ इति [१, २, ३] { प = प + इ। }"), "६");
        assert_eq!(
            shown("मानय प = \"\"। प्रत्येकम् कु इति {\"अ\": १, \"ब\": २} { प = प + कु। }"),
            "अब");
        assert_eq!(
            shown("मानय प = ०। प्रत्येकम् अ इति \"नमः\" { प = प + १। }"), "३");
    }

    #[test]
    fn paridhi_range() {
        assert_eq!(shown("मानय प = ०। प्रत्येकम् इ इति परिधिः(१, १००) { प = प + इ। }"),
                   "५०५०");
        assert_eq!(shown("मानय प = परिधिः(१, ३)।"), "[१, २, ३]");
    }

    // ---- slice 6: classes ----

    #[test]
    fn class_construct_fields_methods() {
        assert_eq!(
            shown("वर्गः छात्रः { विधि आरम्भ(नाम) { अयम्.नाम = नाम। } \
                   विधि परिचय() { फलम् \"अहं \" + अयम्.नाम। } } \
                   मानय र = सृज छात्रः(\"रमा\")। मानय प = र.परिचय()।"),
            "अहं रमा");
    }

    #[test]
    fn class_inheritance() {
        assert_eq!(
            shown("वर्गः व्यक्तिः { विधि नम() { फलम् \"व्यक्तिः\"। } } \
                   वर्गः छात्रः : व्यक्तिः { } \
                   मानय र = सृज छात्रः()। मानय प = र.नम()।"),
            "व्यक्तिः");
    }

    #[test]
    fn class_method_uses_fields_and_lists() {
        assert_eq!(
            shown("वर्गः पेटिका { विधि आरम्भ() { अयम्.वस्तूनि = []। } \
                   विधि योजय_वस्तु(व) { योजय(अयम्.वस्तूनि, व)। } \
                   विधि गणना() { फलम् दैर्घ्यम्(अयम्.वस्तूनि)। } } \
                   मानय प_ = सृज पेटिका()। प_.योजय_वस्तु(\"अ\")। प_.योजय_वस्तु(\"ब\")। \
                   मानय प = प_.गणना()।"),
            "२");
    }

    #[test]
    fn unknown_attribute_errors() {
        assert!(run_ok("वर्गः क { } मानय व = सृज क()। वद(व.अज्ञातम्)।").is_err());
    }

    // ---- slice 6: try/catch ----

    #[test]
    fn try_catches_runtime_error() {
        assert_eq!(
            shown("मानय प = \"\"। प्रयत { मानय क = १ / ०। } दोषे (त्रु) { प = \"गृहीतः\"। }"),
            "गृहीतः");
    }

    #[test]
    fn try_binds_error_message() {
        let out = shown("मानय प = \"\"। प्रयत { वद(अज्ञातनाम)। } दोषे (त्रु) { प = त्रु। }");
        assert!(out.contains("अज्ञातं नाम"), "got: {}", out);
    }

    #[test]
    fn try_without_error_runs_normally() {
        assert_eq!(
            shown("मानय प = १। प्रयत { प = २। } दोषे (त्रु) { प = ३। }"), "२");
    }

    #[test]
    fn program_continues_after_caught_error() {
        assert_eq!(
            shown("मानय प = ०। प्रयत { मानय क = १ / ०। } दोषे (त्रु) { } प = ७।"), "७");
    }

    // ---- recursion guards (unchanged behaviour) ----

    #[test]
    fn deep_recursion_errors_not_crashes() {
        let handle = std::thread::Builder::new()
            .stack_size(8 * 1024 * 1024)
            .spawn(|| {
                let src = "विधि फ(म) { फलम् फ(म + १)। } वद(फ(१))।";
                let toks = lex(src).unwrap();
                let stmts = Parser::new(toks).program().unwrap();
                Interp::new()
                    .with_limits(1_000_000, 2 * 1024 * 1024)
                    .run(&stmts)
            })
            .unwrap();
        let result = handle.join().expect("interpreter thread must not crash");
        assert!(result.is_err());
    }

    // Legitimately deep (but bounded) recursion must work. Note the generous
    // stack: each संस्कृता call costs several native frames, and debug builds
    // make them fat — slice 6's richer Value enum widened them further. The
    // budget is set well inside the thread's stack so the guard, not the OS,
    // decides. (The shipped binary uses 256 MB / 192 MB — see main.rs.)
    #[test]
    fn moderate_recursion_works() {
        let handle = std::thread::Builder::new()
            .stack_size(64 * 1024 * 1024)
            .spawn(|| {
                let src = "विधि गण(म) { यदि (म <= ०) { फलम् ०। } फलम् १ + गण(म - १)। } \
                           मानय प = गण(२००)।";
                let toks = lex(src).unwrap();
                let stmts = Parser::new(toks).program().unwrap();
                let mut it = Interp::new().with_limits(1_000_000, 48 * 1024 * 1024);
                it.run(&stmts)?;
                match it.scopes[0].vars.get("प") {
                    Some(Value::Int(n)) => Ok(n.to_string_signed()),
                    other => Err(format!("unexpected value: {:?}", other)),
                }
            })
            .unwrap();
        let n: Result<String, String> = handle.join().expect("thread must not crash");
        assert_eq!(n.unwrap(), "200");
    }
}
