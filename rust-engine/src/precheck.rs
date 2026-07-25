// precheck.rs — प्राक्परीक्षा, the pre-flight check (वेगः engine).
//
// §2b promises that type annotations are checked BEFORE the program runs, not
// when execution stumbles into them. This pass walks the syntax tree and
// reports every violation it can PROVE from the source alone. It is
// deliberately conservative: it never guesses. Anything it cannot prove is
// left to the runtime check, which still stands behind it.
//
// This mirrors sanskrita.py's `precheck()` statement for statement; the two
// engines must report the same problems for the same program.

use crate::ast::{Expr, Method, Stmt};
use crate::interp::dev_digits;

fn err2(line: usize, sa: &str, en: &str) -> String {
    format!("दोषः पङ्क्तौ {} — {}\nError at line {} — {}",
            dev_digits(&line.to_string()), sa, line, en)
}

/// The type name of an expression whose type is knowable without running the
/// program — or None when it isn't.
fn literal_type(e: &Expr) -> Option<&'static str> {
    match e {
        Expr::Num(s) => Some(if s.contains('.') { "दशमांशः" } else { "पूर्णाङ्कः" }),
        Expr::Str(_) => Some("वाक्यम्"),
        Expr::Bool(_) => Some("सत्यासत्यम्"),
        Expr::Nil => Some("शून्यम्"),
        Expr::List(..) => Some("सूची"),
        Expr::Map(..) => Some("कोशः"),
        Expr::Binary(op, ..) => match op.as_str() {
            "<" | ">" | "<=" | ">=" | "==" | "!=" | "च" | "वा" => Some("सत्यासत्यम्"),
            _ => None,
        },
        Expr::Unary(op, ..) if op == "न" => Some("सत्यासत्यम्"),
        _ => None,
    }
}

fn type_ok(declared: &str, actual: &str) -> bool {
    if declared == "दशमांशः" {
        return actual == "दशमांशः" || actual == "पूर्णाङ्कः";
    }
    declared == actual
}

/// Every problem provable before the program runs. Empty = clean.
pub fn precheck(stmts: &[Stmt]) -> Vec<String> {
    let mut found = Vec::new();
    walk(stmts, &mut found);
    found
}

fn walk(stmts: &[Stmt], found: &mut Vec<String>) {
    for st in stmts {
        match st {
            Stmt::Let { name, expr, ty, nullable, line, .. } => {
                if let Some(tname) = ty {
                    match literal_type(expr) {
                        Some("शून्यम्") if !*nullable => found.push(err2(*line,
                            &format!("'{}' शून्यं न स्वीकरोति — घोषणे '?' प्रयुज्यताम् \
                                      (मानय {}? : {} = शून्यम्।)", name, name, tname),
                            &format!("'{}' cannot hold शून्यम् — declare it nullable \
                                      with '?' (मानय {}? : {} = शून्यम्।)",
                                     name, name, tname))),
                        Some(actual) if actual != "शून्यम्" && !type_ok(tname, actual) =>
                            found.push(err2(*line,
                                &format!("प्रकारदोषः — '{}' {} इति घोषितम्, {} प्राप्तम्",
                                         name, tname, actual),
                                &format!("type error — '{}' is declared {}, got {}",
                                         name, tname, actual))),
                        _ => {}
                    }
                }
                walk_expr(expr, found);
            }
            Stmt::Assign { expr, .. } => walk_expr(expr, found),
            Stmt::ExprStmt(e) => walk_expr(e, found),
            Stmt::If { branches, else_body, .. } => {
                for (cond, body) in branches {
                    walk_expr(cond, found);
                    walk(body, found);
                }
                if let Some(body) = else_body {
                    walk(body, found);
                }
            }
            Stmt::While { cond, body, .. } => { walk_expr(cond, found); walk(body, found); }
            Stmt::ForEach { iter, body, .. } => { walk_expr(iter, found); walk(body, found); }
            Stmt::Func { params, body, .. } => {
                for p in params {
                    if let Some(d) = &p.default {
                        walk_expr(d, found);
                    }
                }
                walk(body, found);
            }
            Stmt::Class { methods, .. } => {
                for Method { body, .. } in methods {
                    walk(body, found);
                }
            }
            Stmt::Try { body, catch, .. } => { walk(body, found); walk(catch, found); }
            Stmt::Throw { expr, .. } => walk_expr(expr, found),
            Stmt::Return { expr: Some(e), .. } => walk_expr(e, found),
            _ => {}
        }
    }
}

fn walk_expr(e: &Expr, found: &mut Vec<String>) {
    match e {
        Expr::Binary(op, l, r, line) => {
            if matches!(op.as_str(), "+" | "-" | "*" | "/" | "%") {
                if let (Some(lt), Some(rt)) = (literal_type(l), literal_type(r)) {
                    let is_text = |t: &str| t == "वाक्यम्";
                    let is_num = |t: &str| t == "पूर्णाङ्कः" || t == "दशमांशः";
                    if is_text(lt) != is_text(rt) && (is_num(lt) || is_num(rt)) {
                        found.push(err2(*line,
                            "वाक्यं सङ्ख्या च न मिश्रणीये — 'वाक्यम्()' प्रयुज्यताम्",
                            "cannot mix text and number — convert with वाक्यम्()/vaakyam()"));
                    } else if op != "+" && (is_text(lt) || is_text(rt)) {
                        found.push(err2(*line, "वाक्येषु एतत् गणितं न शक्यम्",
                                        &format!("'{}' does not work on text", op)));
                    }
                }
            }
            walk_expr(l, found);
            walk_expr(r, found);
        }
        Expr::Unary(_, x, _) => walk_expr(x, found),
        Expr::Call(callee, args, _) => {
            walk_expr(callee, found);
            for a in args {
                walk_expr(&a.value, found);
            }
        }
        Expr::Index(o, i, _) => { walk_expr(o, found); walk_expr(i, found); }
        Expr::Attr(o, _, _) => walk_expr(o, found),
        Expr::List(items, _) => for x in items { walk_expr(x, found); },
        Expr::Map(pairs, _) => for (k, v) in pairs { walk_expr(k, found); walk_expr(v, found); },
        Expr::New(inner, _) => walk_expr(inner, found),
        _ => {}
    }
}

/// Format several problems the way the reference's PrecheckError does.
pub fn report(problems: &[String]) -> String {
    if problems.len() == 1 {
        return problems[0].clone();
    }
    let n = problems.len();
    format!("प्राक्परीक्षायाम् {} दोषाः — कोऽपि आदेशः न चालितः\n\
             {} problems found before running — nothing was executed\n\n{}",
            dev_digits(&n.to_string()), n, problems.join("\n\n"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::lex;
    use crate::parser::Parser;

    fn problems(src: &str) -> Vec<String> {
        let toks = lex(src).unwrap();
        let stmts = Parser::new(toks).program().unwrap();
        precheck(&stmts)
    }

    #[test]
    fn clean_program_has_no_problems() {
        assert!(problems("मानय क : पूर्णाङ्कः = ५। वद(क + १)।").is_empty());
    }

    #[test]
    fn wrong_literal_type_is_caught_before_running() {
        assert_eq!(problems("मानय क : पूर्णाङ्कः = \"पञ्च\"।").len(), 1);
    }

    #[test]
    fn nil_without_question_mark_is_caught() {
        assert_eq!(problems("मानय क : वाक्यम् = शून्यम्।").len(), 1);
        assert!(problems("मानय क? : वाक्यम् = शून्यम्।").is_empty());
    }

    #[test]
    fn mixing_text_and_number_is_caught() {
        assert_eq!(problems("वद(\"अ\" + ५)।").len(), 1);
        assert!(problems("वद(\"अ\" + \"ब\")।").is_empty());
    }

    #[test]
    fn several_problems_are_reported_together() {
        let p = problems("मानय क : वाक्यम् = शून्यम्। वद(\"अ\" + ५)।");
        assert_eq!(p.len(), 2);
        assert!(report(&p).contains("२ दोषाः"));
    }

    // an integer literal satisfies a दशमांशः annotation, as in the reference
    #[test]
    fn int_satisfies_decimal_annotation() {
        assert!(problems("मानय क : दशमांशः = ५।").is_empty());
    }
}
