// parser.rs — recursive-descent parser (वेगः engine)
// Subset: मानय/ध्रुव, assignment, वद & builtin calls, यदि/अथ यदि/अन्यथा,
// यावत्, विरम/अनुवर्त, arithmetic, comparisons, च/वा/न, literals.
// (Functions, lists, classes: later slices.)

use crate::ast::{Arg, Expr, Param, Stmt};
use crate::token::{Tok, Token};

/// Pāṇini's six kārakas — the only valid argument role labels.
const KARAKAS: &[&str] = &["कर्ता", "कर्म", "करण", "सम्प्रदान", "अपादान", "अधिकरण"];

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
            Err(format!("दोषः पङ्क्तौ {} — '{}' अपेक्षितम् / expected '{}'",
                        self.line(), s, s))
        }
    }

    fn eat_end(&mut self) -> PResult<()> {
        if matches!(self.peek().tok, Tok::End) {
            self.advance();
            Ok(())
        } else {
            Err(format!("दोषः पङ्क्तौ {} — दण्डः '।' अपेक्षितः / expected danda",
                        self.line()))
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
                return Err(format!("दोषः पङ्क्तौ {} — '}}' अपेक्षितम्", self.line()));
            }
            stmts.push(self.statement()?);
        }
        self.advance(); // }
        Ok(stmts)
    }

    fn statement(&mut self) -> PResult<Stmt> {
        let line = self.line();
        // declarations
        if self.is_kw("मानय") || self.is_kw("ध्रुव") {
            let is_const = self.is_kw("ध्रुव");
            self.advance();
            let name = self.ident("नाम अपेक्षितम् / expected a name")?;
            self.eat_op("=")?;
            let expr = self.expression()?;
            self.eat_end()?;
            return Ok(Stmt::Let { name, expr, is_const, line });
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
            self.eat_op("(")?;
            let mut params = Vec::new();
            if !self.is_op(")") {
                params.push(self.param()?);
                while self.is_op(",") {
                    self.advance();
                    params.push(self.param()?);
                }
            }
            self.eat_op(")")?;
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
        // assignment:  ident = expr ।   (lookahead)
        if let Tok::Id(name) = &self.peek().tok {
            let name = name.clone();
            if matches!(self.toks.get(self.pos + 1).map(|t| &t.tok),
                        Some(Tok::Op(o)) if o == "=") {
                self.advance(); // id
                self.advance(); // =
                let expr = self.expression()?;
                self.eat_end()?;
                return Ok(Stmt::Assign { name, expr, line });
            }
        }
        // bare expression
        let e = self.expression()?;
        self.eat_end()?;
        Ok(Stmt::ExprStmt(e))
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
                if !self.is_kw("यदि") {
                    return Err(format!("दोषः पङ्क्तौ {} — 'अथ' अनन्तरं 'यदि' अपेक्षितम्",
                                       self.line()));
                }
                self.advance();
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

    /// Parameter: `नाम` or `कर्म नाम` (kāraka role + name).
    fn param(&mut self) -> PResult<Param> {
        let line = self.line();
        let first = self.ident("मापदण्डनाम अपेक्षितम् / expected a parameter name")?;
        if matches!(self.peek().tok, Tok::Id(_)) {
            // two identifiers in a row → the first must be a kāraka
            if !KARAKAS.contains(&first.as_str()) {
                return Err(format!(
                    "दोषः पङ्क्तौ {} — '{}' कारकं न — कारकाणि: कर्ता, कर्म, करण, सम्प्रदान, अपादान, अधिकरण\n\
                     Error at line {} — '{}' is not a kāraka role",
                    line, first, line, first));
            }
            let name = self.ident("मापदण्डनाम अपेक्षितम्")?;
            return Ok(Param { karaka: Some(first), name });
        }
        Ok(Param { karaka: None, name: first })
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
                        "दोषः पङ्क्तौ {} — '{}' कारकं न / '{}' is not a kāraka label",
                        line, label, label));
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
        self.primary()
    }

    fn primary(&mut self) -> PResult<Expr> {
        let line = self.line();
        let t = self.advance();
        match t.tok {
            Tok::Num(s) => Ok(Expr::Num(s)),
            Tok::Str(s) => Ok(Expr::Str(s)),
            Tok::Kw(k) => match k.as_str() {
                "सत्यम्" => Ok(Expr::Bool(true)),
                "असत्यम्" => Ok(Expr::Bool(false)),
                "शून्यम्" => Ok(Expr::Nil),
                _ => Err(format!("दोषः पङ्क्तौ {} — अनपेक्षितः शब्दः '{}'", line, k)),
            },
            Tok::Id(name) => {
                if self.is_op("(") {
                    self.advance(); // (
                    let mut args = Vec::new();
                    if !self.is_op(")") {
                        args.push(self.argument()?);
                        while self.is_op(",") {
                            self.advance();
                            args.push(self.argument()?);
                        }
                    }
                    self.eat_op(")")?;
                    Ok(Expr::Call(name, args, line))
                } else {
                    Ok(Expr::Var(name, line))
                }
            }
            Tok::Op(o) if o == "(" => {
                let e = self.expression()?;
                self.eat_op(")")?;
                Ok(e)
            }
            other => Err(format!("दोषः पङ्क्तौ {} — अनपेक्षितं चिह्नम् '{:?}'", line, other)),
        }
    }
}
