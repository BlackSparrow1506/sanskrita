// parser.rs — recursive-descent parser (वेगः engine), slices 1–6.
//
// Covers: declarations, assignment (name / index / attribute), वद & calls,
// यदि/अथ यदि/अन्यथा, यावत्, प्रत्येकम्…इति, विरम/अनुवर्त, विधि/फलम् with
// kāraka parameters, वर्गः with inheritance, सृज, प्रयत/दोषे, list & map
// literals, indexing, attribute access, and the full expression grammar.

use crate::ast::{Arg, Expr, Method, Param, Stmt, Target};
use crate::token::{Tok, Token};

/// Pāṇini's six kārakas — the only valid argument role labels.
const KARAKAS: &[&str] = &["कर्ता", "कर्म", "करण", "सम्प्रदान", "अपादान", "अधिकरण"];

/// The only names accepted after `:` in a declaration (matches the reference).
const TYPE_NAMES: &[&str] = &["पूर्णाङ्कः", "दशमांशः", "द्रुतदशमांशः", "वाक्यम्",
                              "सत्यासत्यम्", "सूची", "कोशः"];

pub struct Parser {
    toks: Vec<Token>,
    pos: usize,
}

type PResult<T> = Result<T, String>;

impl Parser {
    pub fn new(toks: Vec<Token>) -> Self {
        Parser { toks, pos: 0 }
    }

    fn peek(&self) -> &Token {
        &self.toks[self.pos.min(self.toks.len() - 1)]
    }

    fn line(&self) -> usize {
        self.peek().line
    }

    fn advance(&mut self) -> Token {
        let t = self.toks[self.pos].clone();
        if self.pos < self.toks.len() - 1 {
            self.pos += 1;
        }
        t
    }

    fn is_op(&self, s: &str) -> bool {
        matches!(&self.peek().tok, Tok::Op(o) if o == s)
    }

    fn is_kw(&self, s: &str) -> bool {
        matches!(&self.peek().tok, Tok::Kw(k) if k == s)
    }

    fn eat_op(&mut self, s: &str) -> PResult<()> {
        if self.is_op(s) {
            self.advance();
            Ok(())
        } else {
            Err(format!(
                "दोषः पङ्क्तौ {} — '{}' अपेक्षितम्\nError at line {} — expected '{}'",
                self.line(), s, self.line(), s))
        }
    }

    fn eat_kw(&mut self, s: &str) -> PResult<()> {
        if self.is_kw(s) {
            self.advance();
            Ok(())
        } else {
            Err(format!(
                "दोषः पङ्क्तौ {} — '{}' अपेक्षितम्\nError at line {} — expected '{}'",
                self.line(), s, self.line(), s))
        }
    }

    fn eat_end(&mut self) -> PResult<()> {
        if matches!(self.peek().tok, Tok::End) {
            self.advance();
            Ok(())
        } else if self.is_op("{") {
            Err(format!(
                "दोषः पङ्क्तौ {} — '{{' प्राप्तम् — किं 'यदि' 'यावत्' वा अभिप्रेतम्?\n\
                 Error at line {} — found '{{' — did you mean यदि (if) or यावत् (while)?",
                self.line(), self.line()))
        } else {
            Err(format!(
                "दोषः पङ्क्तौ {} — वाक्यान्ते दण्डः '।' अपेक्षितः\n\
                 Error at line {} — expected danda '।' at end of statement",
                self.line(), self.line()))
        }
    }

    pub fn program(&mut self) -> PResult<Vec<Stmt>> {
        let mut stmts = Vec::new();
        while !matches!(self.peek().tok, Tok::Eof) {
            stmts.push(self.statement()?);
        }
        Ok(stmts)
    }

    fn block(&mut self) -> PResult<Vec<Stmt>> {
        self.eat_op("{")?;
        let mut stmts = Vec::new();
        while !self.is_op("}") {
            if matches!(self.peek().tok, Tok::Eof) {
                return Err(format!(
                    "दोषः पङ्क्तौ {} — '}}' अपेक्षितम्\nError at line {} — expected '}}'",
                    self.line(), self.line()));
            }
            stmts.push(self.statement()?);
        }
        self.advance(); // }
        Ok(stmts)
    }

    fn statement(&mut self) -> PResult<Stmt> {
        let line = self.line();
        if self.is_kw("मानय") || self.is_kw("ध्रुव") {
            let is_const = self.is_kw("ध्रुव");
            self.advance();
            let name = self.ident("नाम अपेक्षितम् / expected a name")?;
            // शून्यम्-safety opt-in: `मानय नाम? : वाक्यम् = शून्यम्।`
            let mut nullable = false;
            if self.is_op("?") {
                self.advance();
                nullable = true;
            }
            // optional type annotation: `मानय क : पूर्णाङ्कः = ५।`
            let mut ty = None;
            if self.is_op(":") {
                self.advance();
                let tline = self.line();
                let t = self.ident("प्रकारः अपेक्षितः / expected a type name")?;
                if !TYPE_NAMES.contains(&t.as_str()) {
                    return Err(format!(
                        "दोषः पङ्क्तौ {} — अज्ञातः प्रकारः '{}' — प्रकाराः: {}\n\
                         Error at line {} — unknown type '{}' — types are: {}",
                        tline, t, TYPE_NAMES.join(", "), tline, t, TYPE_NAMES.join(", ")));
                }
                ty = Some(t);
            }
            self.eat_op("=")?;
            let expr = self.expression()?;
            self.eat_end()?;
            return Ok(Stmt::Let { name, expr, is_const, ty, nullable, line });
        }
        if self.is_kw("विरम") {
            self.advance();
            self.eat_end()?;
            return Ok(Stmt::Break(line));
        }
        if self.is_kw("अनुवर्त") {
            self.advance();
            self.eat_end()?;
            return Ok(Stmt::Continue(line));
        }
        if self.is_kw("विधि") {
            self.advance();
            let name = self.ident("विधिनाम अपेक्षितम् / expected a function name")?;
            let params = self.param_list()?;
            let body = self.block()?;
            return Ok(Stmt::Func { name, params, body, line });
        }
        if self.is_kw("फलम्") {
            self.advance();
            let expr = if matches!(self.peek().tok, Tok::End) {
                None
            } else {
                Some(self.expression()?)
            };
            self.eat_end()?;
            return Ok(Stmt::Return { expr, line });
        }
        if self.is_kw("वर्गः") {
            return self.class_def();
        }
        if self.is_kw("प्रयत") {
            self.advance();
            let body = self.block()?;
            self.eat_kw("दोषे")?;
            self.eat_op("(")?;
            let err_name = self.ident("दोषनाम अपेक्षितम् / expected an error variable")?;
            self.eat_op(")")?;
            let catch = self.block()?;
            return Ok(Stmt::Try { body, err_name, catch, line });
        }
        if self.is_kw("क्षिप") {
            self.advance();
            let expr = self.expression()?;
            self.eat_end()?;
            return Ok(Stmt::Throw { expr, line });
        }
        if self.is_kw("आनय") {
            self.advance();
            let module = match &self.peek().tok {
                Tok::Str(s) => {
                    let s = s.clone();
                    self.advance();
                    s
                }
                _ => return Err(format!(
                    "दोषः पङ्क्तौ {} — आनय-अनन्तरं कोष्ठकनाम अपेक्षितम्\n\
                     Error at line {} — expected a module string after आनय",
                    line, line)),
            };
            self.eat_kw("इति")?;
            let alias = self.ident("नाम अपेक्षितम् / expected a name")?;
            self.eat_end()?;
            return Ok(Stmt::Import { module, alias, line });
        }
        if self.is_kw("प्रत्येकम्") {
            self.advance();
            let var = self.ident("चरनाम अपेक्षितम् / expected a loop variable")?;
            self.eat_kw("इति")?;
            let iter = self.expression()?;
            let body = self.block()?;
            return Ok(Stmt::ForEach { var, iter, body, line });
        }
        if self.is_kw("यदि") {
            return self.if_stmt();
        }
        if self.is_kw("यावत्") {
            self.advance();
            self.eat_op("(")?;
            let cond = self.expression()?;
            self.eat_op(")")?;
            let body = self.block()?;
            return Ok(Stmt::While { cond, body, line });
        }
        // expression statement — or an assignment if '=' follows
        let e = self.expression()?;
        if self.is_op("=") {
            self.advance();
            let value = self.expression()?;
            self.eat_end()?;
            let target = match e {
                Expr::Var(n, _) => Target::Var(n),
                Expr::Index(obj, idx, _) => Target::Index(*obj, *idx),
                Expr::Attr(obj, name, _) => Target::Attr(*obj, name),
                _ => return Err(format!(
                    "दोषः पङ्क्तौ {} — एतस्मै मूल्यं दातुं न शक्यम्\n\
                     Error at line {} — cannot assign to this expression", line, line)),
            };
            return Ok(Stmt::Assign { target, expr: value, line });
        }
        self.eat_end()?;
        Ok(Stmt::ExprStmt(e))
    }

    fn class_def(&mut self) -> PResult<Stmt> {
        let line = self.line();
        self.advance(); // वर्गः
        let name = self.ident("वर्गनाम अपेक्षितम् / expected a class name")?;
        let mut parent = None;
        if self.is_op(":") {
            self.advance();
            parent = Some(self.ident("मातृवर्गनाम अपेक्षितम् / expected a parent class")?);
        }
        self.eat_op("{")?;
        let mut methods = Vec::new();
        while !self.is_op("}") {
            if matches!(self.peek().tok, Tok::Eof) {
                return Err(format!(
                    "दोषः पङ्क्तौ {} — '}}' अपेक्षितम् वर्गान्ते\n\
                     Error at line {} — expected '}}' to close the class",
                    self.line(), self.line()));
            }
            self.eat_kw("विधि").map_err(|_| format!(
                "दोषः पङ्क्तौ {} — वर्गे केवलं 'विधि' लेख्याः\n\
                 Error at line {} — only विधि (methods) are allowed inside a वर्गः",
                self.line(), self.line()))?;
            let mname = self.ident("विधिनाम अपेक्षितम् / expected a method name")?;
            let params = self.param_list()?;
            let body = self.block()?;
            methods.push(Method { name: mname, params, body });
        }
        self.advance(); // }
        Ok(Stmt::Class { name, parent, methods, line })
    }

    fn param_list(&mut self) -> PResult<Vec<Param>> {
        self.eat_op("(")?;
        let mut params = Vec::new();
        if !self.is_op(")") {
            params.push(self.param()?);
            while self.is_op(",") {
                self.advance();
                if self.is_op(")") { break; }              // trailing comma
                params.push(self.param()?);
            }
        }
        self.eat_op(")")?;
        Ok(params)
    }

    /// Parameter: `नाम` or `कर्म नाम` (kāraka role + name).
    fn param(&mut self) -> PResult<Param> {
        let line = self.line();
        let first = self.ident("मापदण्डनाम अपेक्षितम् / expected a parameter name")?;
        let mut karaka = None;
        let mut name = first;
        if matches!(self.peek().tok, Tok::Id(_)) {
            if !KARAKAS.contains(&name.as_str()) {
                return Err(format!(
                    "दोषः पङ्क्तौ {} — '{}' कारकं न — कारकाणि: कर्ता, कर्म, करण, सम्प्रदान, अपादान, अधिकरण\n\
                     Error at line {} — '{}' is not a kāraka role", line, name, line, name));
            }
            karaka = Some(name);
            name = self.ident("मापदण्डनाम अपेक्षितम्")?;
        }
        let mut default = None;
        if self.is_op("=") {
            self.advance();
            default = Some(self.expression()?);
        }
        Ok(Param { karaka, name, default })
    }

    /// Argument: `मूल्यम्` or `कर्म: मूल्यम्` (kāraka-labeled).
    fn argument(&mut self) -> PResult<Arg> {
        if let Tok::Id(label) = &self.peek().tok {
            let label = label.clone();
            if matches!(self.toks.get(self.pos + 1).map(|t| &t.tok),
                        Some(Tok::Op(o)) if o == ":") {
                let line = self.line();
                if !KARAKAS.contains(&label.as_str()) {
                    return Err(format!(
                        "दोषः पङ्क्तौ {} — '{}' कारकं न\n\
                         Error at line {} — '{}' is not a kāraka label",
                        line, label, line, label));
                }
                self.advance(); // label
                self.advance(); // :
                let value = self.expression()?;
                return Ok(Arg { karaka: Some(label), value });
            }
        }
        Ok(Arg { karaka: None, value: self.expression()? })
    }

    fn ident(&mut self, msg: &str) -> PResult<String> {
        if let Tok::Id(n) = &self.peek().tok {
            let n = n.clone();
            self.advance();
            Ok(n)
        } else {
            Err(format!("दोषः पङ्क्तौ {} — {}", self.line(), msg))
        }
    }

    fn if_stmt(&mut self) -> PResult<Stmt> {
        let line = self.line();
        self.advance(); // यदि
        self.eat_op("(")?;
        let cond = self.expression()?;
        self.eat_op(")")?;
        let body = self.block()?;
        let mut branches = vec![(cond, body)];
        let mut else_body = None;
        loop {
            if self.is_kw("अथ") {
                self.advance();
                self.eat_kw("यदि")?;
                self.eat_op("(")?;
                let c = self.expression()?;
                self.eat_op(")")?;
                let b = self.block()?;
                branches.push((c, b));
            } else if self.is_kw("अन्यथा") {
                self.advance();
                else_body = Some(self.block()?);
                break;
            } else {
                break;
            }
        }
        Ok(Stmt::If { branches, else_body, line })
    }

    // ---- expressions (precedence climbing) ----

    fn expression(&mut self) -> PResult<Expr> {
        self.or_expr()
    }

    fn or_expr(&mut self) -> PResult<Expr> {
        let mut left = self.and_expr()?;
        while self.is_kw("वा") {
            let line = self.line();
            self.advance();
            let right = self.and_expr()?;
            left = Expr::Binary("वा".into(), Box::new(left), Box::new(right), line);
        }
        Ok(left)
    }

    fn and_expr(&mut self) -> PResult<Expr> {
        let mut left = self.not_expr()?;
        while self.is_kw("च") {
            let line = self.line();
            self.advance();
            let right = self.not_expr()?;
            left = Expr::Binary("च".into(), Box::new(left), Box::new(right), line);
        }
        Ok(left)
    }

    fn not_expr(&mut self) -> PResult<Expr> {
        if self.is_kw("न") {
            let line = self.line();
            self.advance();
            let sub = self.not_expr()?;
            return Ok(Expr::Unary("न".into(), Box::new(sub), line));
        }
        self.comparison()
    }

    fn comparison(&mut self) -> PResult<Expr> {
        let left = self.additive()?;
        if let Tok::Op(o) = &self.peek().tok {
            if ["==", "!=", "<", ">", "<=", ">="].contains(&o.as_str()) {
                let op = o.clone();
                let line = self.line();
                self.advance();
                let right = self.additive()?;
                return Ok(Expr::Binary(op, Box::new(left), Box::new(right), line));
            }
        }
        Ok(left)
    }

    fn additive(&mut self) -> PResult<Expr> {
        let mut left = self.multiplicative()?;
        loop {
            if let Tok::Op(o) = &self.peek().tok {
                if o == "+" || o == "-" {
                    let op = o.clone();
                    let line = self.line();
                    self.advance();
                    let right = self.multiplicative()?;
                    left = Expr::Binary(op, Box::new(left), Box::new(right), line);
                    continue;
                }
            }
            break;
        }
        Ok(left)
    }

    fn multiplicative(&mut self) -> PResult<Expr> {
        let mut left = self.unary()?;
        loop {
            if let Tok::Op(o) = &self.peek().tok {
                if o == "*" || o == "/" || o == "%" {
                    let op = o.clone();
                    let line = self.line();
                    self.advance();
                    let right = self.unary()?;
                    left = Expr::Binary(op, Box::new(left), Box::new(right), line);
                    continue;
                }
            }
            break;
        }
        Ok(left)
    }

    fn unary(&mut self) -> PResult<Expr> {
        if self.is_op("-") {
            let line = self.line();
            self.advance();
            let sub = self.unary()?;
            return Ok(Expr::Unary("-".into(), Box::new(sub), line));
        }
        let base = self.primary()?;
        self.postfix(base)
    }

    /// Chained calls, indexing and attribute access: `अ.ब(क)[ख].ग`
    fn postfix(&mut self, mut e: Expr) -> PResult<Expr> {
        loop {
            let line = self.line();
            if self.is_op("(") {
                self.advance();
                let mut args = Vec::new();
                if !self.is_op(")") {
                    args.push(self.argument()?);
                    while self.is_op(",") {
                        self.advance();
                        if self.is_op(")") { break; }      // trailing comma
                        args.push(self.argument()?);
                    }
                }
                self.eat_op(")")?;
                e = Expr::Call(Box::new(e), args, line);
            } else if self.is_op("[") {
                self.advance();
                let idx = self.expression()?;
                self.eat_op("]")?;
                e = Expr::Index(Box::new(e), Box::new(idx), line);
            } else if self.is_op(".") {
                self.advance();
                let name = self.ident("नाम अपेक्षितम् '.' अनन्तरम् / expected a name after '.'")?;
                e = Expr::Attr(Box::new(e), name, line);
            } else {
                break;
            }
        }
        Ok(e)
    }

    fn primary(&mut self) -> PResult<Expr> {
        let line = self.line();
        // list literal
        if self.is_op("[") {
            self.advance();
            let mut items = Vec::new();
            if !self.is_op("]") {
                items.push(self.expression()?);
                while self.is_op(",") {
                    self.advance();
                    if self.is_op("]") { break; }          // trailing comma
                    items.push(self.expression()?);
                }
            }
            self.eat_op("]")?;
            return Ok(Expr::List(items, line));
        }
        // map literal
        if self.is_op("{") {
            self.advance();
            let mut pairs = Vec::new();
            if !self.is_op("}") {
                loop {
                    let k = self.expression()?;
                    self.eat_op(":")?;
                    let v = self.expression()?;
                    pairs.push((k, v));
                    if self.is_op(",") {
                        self.advance();
                        if self.is_op("}") { break; }      // trailing comma
                        continue;
                    }
                    break;
                }
            }
            self.eat_op("}")?;
            return Ok(Expr::Map(pairs, line));
        }
        let t = self.advance();
        match t.tok {
            Tok::Num(s) => Ok(Expr::Num(s)),
            Tok::Str(s) => Ok(Expr::Str(s)),
            Tok::Kw(k) => match k.as_str() {
                "सत्यम्" => Ok(Expr::Bool(true)),
                "असत्यम्" => Ok(Expr::Bool(false)),
                "शून्यम्" => Ok(Expr::Nil),
                "अयम्" => Ok(Expr::Var("अयम्".into(), line)),
                "सृज" => {
                    let base = self.primary()?;
                    let inner = self.postfix(base)?;
                    Ok(Expr::New(Box::new(inner), line))
                }
                _ => Err(format!(
                    "दोषः पङ्क्तौ {} — अनपेक्षितः शब्दः '{}'\n\
                     Error at line {} — unexpected keyword '{}'", line, k, line, k)),
            },
            Tok::Id(name) => Ok(Expr::Var(name, line)),
            Tok::Op(o) if o == "(" => {
                let e = self.expression()?;
                self.eat_op(")")?;
                Ok(e)
            }
            other => Err(format!(
                "दोषः पङ्क्तौ {} — अनपेक्षितं चिह्नम् '{:?}'\n\
                 Error at line {} — unexpected token", line, other, line)),
        }
    }
}
