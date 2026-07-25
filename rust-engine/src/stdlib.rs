// stdlib.rs — native संस्कृता modules for the वेगः engine.
//
// संस्कृतम्   — linguistics (sandhi, meter, syllables, transliteration)
// गणितम्      — mathematics (Āryabhaṭa's ज्या for sine, etc.)
// वाक्यकर्म   — text operations
// यादृच्छिकम् — randomness
// कालः        — time
//
// Each function mirrors the Python reference so both engines agree.

use crate::bigint::BigInt;
use crate::decimal::Decimal;
use crate::interp::{dev_digits, Interp};
use crate::sanskritam;
use crate::value::Value;

type RResult<T> = Result<T, String>;

const SANSKRITAM: &[&str] = &["अक्षराणि", "अक्षरगणना", "मात्राः", "छन्दः",
                              "रोमनय", "देवनागरय", "संधय"];
const GANITAM: &[&str] = &["वर्गमूलम्", "घातः", "तलम्", "उपरितलम्", "निरपेक्षम्",
                           "ज्या", "कोज्या", "पाई", "ई"];
const VAKYAKARMA: &[&str] = &["विभज", "संयोजय", "खोज", "प्रतिस्थापय", "अंश",
                              "उच्च", "निम्न", "परिष्कार", "आरभते", "अन्तयति",
                              "अन्तर्भवति"];
const YADRCCHIKAM: &[&str] = &["अन्तरे", "वरय", "भिन्नम्"];
const KALAH: &[&str] = &["अद्य", "संप्रति", "वर्षः", "क्षणविरामः"];
const SUCHIKARMA: &[&str] = &["छानय", "प्रतिचित्रय", "न्यूनीकरण", "विपर्यय",
                              "अन्तर्भवति", "अनुक्रमः", "योगः", "महत्तमम्",
                              "लघुत्तमम्", "अद्वितीयम्"];
const SANCHIKA: &[&str] = &["पठ", "लिख", "योजय", "अस्ति", "निष्कासय",
                            "पङ्क्तयः", "सूचिका"];
const JSON: &[&str] = &["विश्लेषय", "पाठय"];

fn members(module: &str) -> &'static [&'static str] {
    match module {
        "संस्कृतम्" => SANSKRITAM,
        "गणितम्" => GANITAM,
        "वाक्यकर्म" => VAKYAKARMA,
        "यादृच्छिकम्" => YADRCCHIKAM,
        "कालः" => KALAH,
        "सूचीकर्म" => SUCHIKARMA,
        "सञ्चिका" => SANCHIKA,
        "जेसन" => JSON,
        _ => &[],
    }
}

// §7d #2 — "sandhi-style composition": chain operations the way Sanskrit
// compounds words. `.नाम` on a सूची / कोशः / वाक्यम् resolves to the SAME
// stdlib function you would call as सू.नाम(सूची, …), with the receiver passed
// as the first argument. One implementation, two spellings.
const LIST_METHODS: &[&str] = SUCHIKARMA;
const STR_METHODS: &[&str] = VAKYAKARMA;
/// Builtins that also read naturally as methods, per receiver type.
const LIST_BUILTINS: &[&str] = &["क्रमय", "दैर्घ्यम्", "योजय", "अपनय"];
const MAP_BUILTINS: &[&str] = &["कुञ्जिकाः", "दैर्घ्यम्", "अपनय"];
const STR_BUILTINS: &[&str] = &["दैर्घ्यम्"];

/// Resolve `receiver.name`. Returns (module, interned name); module is None
/// when the name is a builtin rather than a stdlib module function.
pub fn method_for(type_name: &str, name: &str)
    -> Option<(Option<&'static str>, &'static str)>
{
    const NONE: &[&str] = &[];
    let (mod_name, mod_members, builtins): (&'static str,
                                            &'static [&'static str],
                                            &'static [&'static str]) =
        match type_name {
            "सूची" => ("सूचीकर्म", LIST_METHODS, LIST_BUILTINS),
            "वाक्यम्" => ("वाक्यकर्म", STR_METHODS, STR_BUILTINS),
            "कोशः" => ("", NONE, MAP_BUILTINS),
            _ => return None,
        };
    if let Some(n) = mod_members.iter().find(|m| **m == name) {
        return Some((Some(mod_name), *n));
    }
    builtins.iter().find(|m| **m == name).map(|n| (None, *n))
}

/// Every method name a receiver type understands — used for "did you mean?".
pub fn methods_of(type_name: &str) -> Vec<&'static str> {
    let mut out: Vec<&'static str> = match type_name {
        "सूची" => LIST_METHODS.iter().chain(LIST_BUILTINS).copied().collect(),
        "वाक्यम्" => STR_METHODS.iter().chain(STR_BUILTINS).copied().collect(),
        "कोशः" => MAP_BUILTINS.to_vec(),
        _ => Vec::new(),
    };
    out.sort();
    out.dedup();
    out
}

pub fn has(module: &str, name: &str) -> bool {
    members(module).contains(&name)
}

/// Members that are CONSTANTS, not functions: `ग.पाई` yields the number
/// directly (no parentheses), exactly as in the reference.
pub fn is_const(module: &str, name: &str) -> bool {
    matches!((module, name), ("गणितम्", "पाई") | ("गणितम्", "ई"))
}

/// Return the &'static str for a member name (so Value::Native can hold it).
pub fn intern(module: &str, name: &str) -> &'static str {
    members(module).iter().find(|m| **m == name).copied().unwrap_or("")
}

fn err(line: usize, sa: &str, en: &str) -> String {
    format!("दोषः पङ्क्तौ {} — {}\nError at line {} — {}",
            dev_digits(&line.to_string()), sa, line, en)
}

fn want_str(v: Option<&Value>, line: usize, who: &str) -> RResult<String> {
    match v {
        Some(Value::Str(s)) => Ok(s.clone()),
        _ => Err(err(line, &format!("{} वाक्यम् अपेक्षते", who),
                     &format!("{} expects text", who))),
    }
}

fn want_num(v: Option<&Value>, line: usize, who: &str) -> RResult<f64> {
    match v {
        Some(x) if x.is_number() => {
            let s = x.as_decimal().map(|d| d.to_plain_string()).unwrap_or_default();
            s.parse::<f64>().map_err(|_| err(line,
                &format!("{} सङ्ख्याम् अपेक्षते", who),
                &format!("{} expects a number", who)))
        }
        _ => Err(err(line, &format!("{} सङ्ख्याम् अपेक्षते", who),
                     &format!("{} expects a number", who))),
    }
}

fn want_list(v: Option<&Value>, line: usize, who: &str) -> RResult<Vec<Value>> {
    match v {
        Some(Value::List(l)) => Ok(l.borrow().clone()),
        _ => Err(err(line, &format!("{} सूचीम् अपेक्षते", who),
                     &format!("{} expects a list", who))),
    }
}

fn want_int(v: Option<&Value>, line: usize, who: &str) -> RResult<i64> {
    match v {
        Some(Value::Int(b)) => b.to_i64().ok_or_else(|| err(line,
            &format!("{} अतिविशाला सङ्ख्या", who),
            &format!("{} got a number too large", who))),
        _ => Err(err(line, &format!("{} पूर्णाङ्कम् अपेक्षते", who),
                     &format!("{} expects a whole number", who))),
    }
}

/// Convert an f64 result to an exact decimal value (matching how the reference
/// surfaces Python floats through the bridge: as decimals).
fn num(x: f64) -> Value {
    let s = format!("{}", x);
    match Decimal::parse(&s) {
        Some(d) => Value::Dec(d),
        None => Value::Str(s),
    }
}

pub fn call(it: &mut Interp, module: &str, name: &str, vals: Vec<Value>, line: usize)
    -> RResult<Value>
{
    match (module, name) {
        // ---------------- संस्कृतम् ----------------
        ("संस्कृतम्", "अक्षराणि") => {
            let t = want_str(vals.first(), line, "अक्षराणि()")?;
            Ok(Value::list(sanskritam::aksharani(&t).into_iter()
                .map(Value::Str).collect()))
        }
        ("संस्कृतम्", "अक्षरगणना") => {
            let t = want_str(vals.first(), line, "अक्षरगणना()")?;
            Ok(Value::int(sanskritam::aksharani(&t).len() as i64))
        }
        ("संस्कृतम्", "मात्राः") => {
            let t = want_str(vals.first(), line, "मात्राः()")?;
            Ok(Value::list(sanskritam::matrah(&t).into_iter()
                .map(Value::Str).collect()))
        }
        ("संस्कृतम्", "छन्दः") => {
            let t = want_str(vals.first(), line, "छन्दः()")?;
            Ok(Value::Str(sanskritam::chandah(&t)))
        }
        ("संस्कृतम्", "रोमनय") => {
            let t = want_str(vals.first(), line, "रोमनय()")?;
            Ok(Value::Str(sanskritam::romanaya(&t)))
        }
        ("संस्कृतम्", "देवनागरय") => {
            let t = want_str(vals.first(), line, "देवनागरय()")?;
            Ok(Value::Str(sanskritam::devanagaraya(&t)))
        }
        ("संस्कृतम्", "संधय") => {
            let a = want_str(vals.first(), line, "संधय()")?;
            let b = want_str(vals.get(1), line, "संधय()")?;
            Ok(Value::Str(sanskritam::sandhaya(&a, &b)))
        }

        // ---------------- गणितम् ----------------
        ("गणितम्", "पाई") => Ok(num(std::f64::consts::PI)),
        ("गणितम्", "ई") => Ok(num(std::f64::consts::E)),
        ("गणितम्", "वर्गमूलम्") => Ok(num(want_num(vals.first(), line, "वर्गमूलम्()")?.sqrt())),
        ("गणितम्", "घातः") => {
            let b = want_num(vals.first(), line, "घातः()")?;
            let e = want_num(vals.get(1), line, "घातः()")?;
            Ok(num(b.powf(e)))
        }
        ("गणितम्", "ज्या") => Ok(num(want_num(vals.first(), line, "ज्या()")?.sin())),
        ("गणितम्", "कोज्या") => Ok(num(want_num(vals.first(), line, "कोज्या()")?.cos())),
        ("गणितम्", "तलम्") => {
            let x = want_num(vals.first(), line, "तलम्()")?;
            Ok(Value::Int(BigInt::from_i64(x.floor() as i64)))
        }
        ("गणितम्", "उपरितलम्") => {
            let x = want_num(vals.first(), line, "उपरितलम्()")?;
            Ok(Value::Int(BigInt::from_i64(x.ceil() as i64)))
        }
        ("गणितम्", "निरपेक्षम्") => match vals.first() {
            Some(Value::Int(b)) => Ok(Value::Int(b.abs())),
            Some(Value::Dec(d)) => Ok(Value::Dec(
                if d.to_plain_string().starts_with('-') { d.neg() } else { d.clone() })),
            _ => Err(err(line, "निरपेक्षम्() सङ्ख्याम् अपेक्षते",
                         "निरपेक्षम्() expects a number")),
        },

        // ---------------- वाक्यकर्म ----------------
        ("वाक्यकर्म", "विभज") => {
            let t = want_str(vals.first(), line, "विभज()")?;
            let sep = want_str(vals.get(1), line, "विभज()")?;
            let parts: Vec<Value> = if sep.is_empty() {
                t.chars().map(|c| Value::Str(c.to_string())).collect()
            } else {
                t.split(sep.as_str()).map(|p| Value::Str(p.to_string())).collect()
            };
            Ok(Value::list(parts))
        }
        ("वाक्यकर्म", "संयोजय") => {
            let sep = want_str(vals.get(1), line, "संयोजय()")?;
            match vals.first() {
                Some(Value::List(l)) => {
                    let mut parts = Vec::new();
                    for v in l.borrow().iter() {
                        match v {
                            Value::Str(s) => parts.push(s.clone()),
                            other => parts.push(it.display(other)),
                        }
                    }
                    Ok(Value::Str(parts.join(&sep)))
                }
                _ => Err(err(line, "संयोजय() सूचीम् अपेक्षते",
                             "संयोजय() expects a list")),
            }
        }
        ("वाक्यकर्म", "खोज") => {
            let t = want_str(vals.first(), line, "खोज()")?;
            let sub = want_str(vals.get(1), line, "खोज()")?;
            // 1-based character position; ० when absent (matches the reference)
            let pos = match t.find(sub.as_str()) {
                Some(byte_idx) => t[..byte_idx].chars().count() as i64 + 1,
                None => 0,
            };
            Ok(Value::int(pos))
        }
        ("वाक्यकर्म", "प्रतिस्थापय") => {
            let t = want_str(vals.first(), line, "प्रतिस्थापय()")?;
            let from = want_str(vals.get(1), line, "प्रतिस्थापय()")?;
            let to = want_str(vals.get(2), line, "प्रतिस्थापय()")?;
            Ok(Value::Str(t.replace(from.as_str(), to.as_str())))
        }
        ("वाक्यकर्म", "अंश") => {
            let t = want_str(vals.first(), line, "अंश()")?;
            let i = want_int(vals.get(1), line, "अंश()")?;
            let j = want_int(vals.get(2), line, "अंश()")?;
            let chars: Vec<char> = t.chars().collect();
            let start = (i.max(1) as usize).saturating_sub(1);
            let end = (j.max(0) as usize).min(chars.len());
            if start >= end {
                return Ok(Value::Str(String::new()));
            }
            Ok(Value::Str(chars[start..end].iter().collect()))
        }

        // ---------------- यादृच्छिकम् ----------------
        // Deterministic-free randomness without dependencies: seed from the
        // clock and advance an xorshift generator.
        ("यादृच्छिकम्", "अन्तरे") => {
            let lo = want_int(vals.first(), line, "अन्तरे()")?;
            let hi = want_int(vals.get(1), line, "अन्तरे()")?;
            if hi < lo {
                return Err(err(line, "अन्तरे(आदिः, अन्तः) — अन्तः न्यूनः",
                               "अन्तरे(low, high) — high is less than low"));
            }
            let span = (hi - lo + 1) as u64;
            Ok(Value::int(lo + (next_rand() % span) as i64))
        }
        ("यादृच्छिकम्", "वरय") => match vals.first() {
            Some(Value::List(l)) => {
                let items = l.borrow();
                if items.is_empty() {
                    return Err(err(line, "वरय() रिक्तायाः सूच्याः न शक्यम्",
                                   "वरय() cannot choose from an empty list"));
                }
                let idx = (next_rand() % items.len() as u64) as usize;
                Ok(items[idx].clone())
            }
            _ => Err(err(line, "वरय() सूचीम् अपेक्षते", "वरय() expects a list")),
        },
        ("यादृच्छिकम्", "भिन्नम्") => {
            let r = (next_rand() % 1_000_000_000) as f64 / 1_000_000_000.0;
            Ok(num(r))
        }

        // ---------------- वाक्यकर्म (continued) ----------------
        ("वाक्यकर्म", "उच्च") =>
            Ok(Value::Str(want_str(vals.first(), line, "उच्च()")?.to_uppercase())),
        ("वाक्यकर्म", "निम्न") =>
            Ok(Value::Str(want_str(vals.first(), line, "निम्न()")?.to_lowercase())),
        ("वाक्यकर्म", "परिष्कार") =>
            Ok(Value::Str(want_str(vals.first(), line, "परिष्कार()")?.trim().to_string())),
        ("वाक्यकर्म", "आरभते") => {
            let t = want_str(vals.first(), line, "आरभते()")?;
            let p = want_str(vals.get(1), line, "आरभते()")?;
            Ok(Value::Bool(t.starts_with(p.as_str())))
        }
        ("वाक्यकर्म", "अन्तयति") => {
            let t = want_str(vals.first(), line, "अन्तयति()")?;
            let p = want_str(vals.get(1), line, "अन्तयति()")?;
            Ok(Value::Bool(t.ends_with(p.as_str())))
        }
        ("वाक्यकर्म", "अन्तर्भवति") => {
            let t = want_str(vals.first(), line, "अन्तर्भवति()")?;
            let p = want_str(vals.get(1), line, "अन्तर्भवति()")?;
            Ok(Value::Bool(t.contains(p.as_str())))
        }

        // ---------------- सूचीकर्म ----------------
        ("सूचीकर्म", "छानय") => {
            let items = want_list(vals.first(), line, "छानय()")?;
            let f = vals.get(1).cloned().ok_or_else(|| err(line,
                "छानय(सूची, विधिः)", "छानय(list, function)"))?;
            let mut out = Vec::new();
            for v in items {
                match it.apply(&f, vec![v.clone()], line)? {
                    Value::Bool(true) => out.push(v),
                    Value::Bool(false) => {}
                    other => return Err(err(line,
                        &format!("छानय() सत्यासत्यम् अपेक्षते, {} प्राप्तम्", other.type_name()),
                        "छानय() expects the function to return सत्यम्/असत्यम्")),
                }
            }
            Ok(Value::list(out))
        }
        ("सूचीकर्म", "प्रतिचित्रय") => {
            let items = want_list(vals.first(), line, "प्रतिचित्रय()")?;
            let f = vals.get(1).cloned().ok_or_else(|| err(line,
                "प्रतिचित्रय(सूची, विधिः)", "प्रतिचित्रय(list, function)"))?;
            let mut out = Vec::new();
            for v in items {
                out.push(it.apply(&f, vec![v], line)?);
            }
            Ok(Value::list(out))
        }
        ("सूचीकर्म", "न्यूनीकरण") => {
            let items = want_list(vals.first(), line, "न्यूनीकरण()")?;
            let f = vals.get(1).cloned().ok_or_else(|| err(line,
                "न्यूनीकरण(सूची, विधिः, आदिः)", "न्यूनीकरण(list, function, initial)"))?;
            let mut acc = vals.get(2).cloned().ok_or_else(|| err(line,
                "न्यूनीकरण(सूची, विधिः, आदिः)", "न्यूनीकरण(list, function, initial)"))?;
            for v in items {
                acc = it.apply(&f, vec![acc, v], line)?;
            }
            Ok(acc)
        }
        ("सूचीकर्म", "विपर्यय") => {
            let mut items = want_list(vals.first(), line, "विपर्यय()")?;
            items.reverse();
            Ok(Value::list(items))
        }
        ("सूचीकर्म", "अन्तर्भवति") => {
            let items = want_list(vals.first(), line, "अन्तर्भवति()")?;
            let needle = vals.get(1).cloned().unwrap_or(Value::Nil);
            Ok(Value::Bool(items.iter().any(|x| *x == needle)))
        }
        ("सूचीकर्म", "अनुक्रमः") => {
            let items = want_list(vals.first(), line, "अनुक्रमः()")?;
            let needle = vals.get(1).cloned().unwrap_or(Value::Nil);
            // 1-based, ० when absent — the same convention as वाक्यकर्म.खोज
            let pos = items.iter().position(|x| *x == needle)
                .map(|i| i as i64 + 1).unwrap_or(0);
            Ok(Value::int(pos))
        }
        ("सूचीकर्म", "योगः") => {
            let items = want_list(vals.first(), line, "योगः()")?;
            let mut acc = Value::int(0);
            for v in &items {
                acc = it.add_values(&acc, v, line)?;
            }
            Ok(acc)
        }
        ("सूचीकर्म", "महत्तमम्") | ("सूचीकर्म", "लघुत्तमम्") => {
            let who = if name == "महत्तमम्" { "महत्तमम्()" } else { "लघुत्तमम्()" };
            let items = want_list(vals.first(), line, who)?;
            if items.is_empty() {
                return Err(err(line, &format!("{} रिक्तायाः सूच्याः न शक्यम्", who),
                               &format!("{} of an empty list", who)));
            }
            let want_max = name == "महत्तमम्";
            let mut best = items[0].clone();
            for v in items.iter().skip(1) {
                let greater = it.greater_than(v, &best, line)?;
                if greater == want_max {
                    best = v.clone();
                }
            }
            Ok(best)
        }
        ("सूचीकर्म", "अद्वितीयम्") => {
            let items = want_list(vals.first(), line, "अद्वितीयम्()")?;
            let mut out: Vec<Value> = Vec::new();
            for v in items {
                if !out.iter().any(|x| *x == v) {
                    out.push(v);
                }
            }
            Ok(Value::list(out))
        }

        // ---------------- सञ्चिका (files — UTF-8, always) ----------------
        ("सञ्चिका", "पठ") => {
            let path = want_str(vals.first(), line, "पठ()")?;
            std::fs::read_to_string(&path).map(Value::Str)
                .map_err(|e| err(line,
                    &format!("'{}' सञ्चिका न पठ्यते", path),
                    &format!("cannot read file '{}': {}", path, e)))
        }
        ("सञ्चिका", "लिख") | ("सञ्चिका", "योजय") => {
            let who = if name == "लिख" { "लिख()" } else { "योजय()" };
            let path = want_str(vals.first(), line, who)?;
            let text = want_str(vals.get(1), line, who)?;
            let res = if name == "लिख" {
                std::fs::write(&path, text.as_bytes())
            } else {
                use std::io::Write;
                std::fs::OpenOptions::new().create(true).append(true).open(&path)
                    .and_then(|mut f| f.write_all(text.as_bytes()))
            };
            res.map(|_| Value::Nil).map_err(|e| err(line,
                &format!("'{}' सञ्चिकायां न लिख्यते", path),
                &format!("cannot write file '{}': {}", path, e)))
        }
        ("सञ्चिका", "अस्ति") => {
            let path = want_str(vals.first(), line, "अस्ति()")?;
            Ok(Value::Bool(std::path::Path::new(&path).exists()))
        }
        ("सञ्चिका", "निष्कासय") => {
            let path = want_str(vals.first(), line, "निष्कासय()")?;
            std::fs::remove_file(&path).map(|_| Value::Nil).map_err(|e| err(line,
                &format!("'{}' सञ्चिका न निष्कास्यते", path),
                &format!("cannot remove file '{}': {}", path, e)))
        }
        ("सञ्चिका", "पङ्क्तयः") => {
            let path = want_str(vals.first(), line, "पङ्क्तयः()")?;
            let text = std::fs::read_to_string(&path).map_err(|e| err(line,
                &format!("'{}' सञ्चिका न पठ्यते", path),
                &format!("cannot read file '{}': {}", path, e)))?;
            Ok(Value::list(text.lines().map(|l| Value::Str(l.to_string())).collect()))
        }
        ("सञ्चिका", "सूचिका") => {
            let path = want_str(vals.first(), line, "सूचिका()")?;
            let rd = std::fs::read_dir(&path).map_err(|e| err(line,
                &format!("'{}' सूचिका न पठ्यते", path),
                &format!("cannot list directory '{}': {}", path, e)))?;
            let mut names: Vec<String> = rd.filter_map(|e| e.ok())
                .map(|e| e.file_name().to_string_lossy().to_string()).collect();
            names.sort();
            Ok(Value::list(names.into_iter().map(Value::Str).collect()))
        }

        // ---------------- जेसन ----------------
        ("जेसन", "विश्लेषय") => {
            let t = want_str(vals.first(), line, "विश्लेषय()")?;
            json_parse(&t, line)
        }
        ("जेसन", "पाठय") => {
            let v = vals.first().cloned().unwrap_or(Value::Nil);
            let mut out = String::new();
            json_write(&v, &mut out, line)?;
            Ok(Value::Str(out))
        }

        // ---------------- कालः ----------------
        ("कालः", "अद्य") => Ok(Value::Str(dev_digits(&today()))),
        ("कालः", "संप्रति") => Ok(Value::Str(dev_digits(&clock()))),
        ("कालः", "वर्षः") => Ok(Value::int(year())),
        ("कालः", "क्षणविरामः") => {
            let secs = want_num(vals.first(), line, "क्षणविरामः()")?;
            if secs > 0.0 {
                std::thread::sleep(std::time::Duration::from_secs_f64(secs));
            }
            Ok(Value::Nil)
        }

        _ => Err(err(line,
            &format!("'{}' कोष्ठके '{}' नास्ति", module, name),
            &format!("module '{}' has no '{}'", module, name))),
    }
}

// ---- जेसन: a dependency-free JSON codec ----
//
// Mirrors the reference (`json.dumps(..., ensure_ascii=False)` / `json.loads`):
// whole numbers stay पूर्णाङ्कः, numbers with a fraction or exponent become
// दशमांशः — never a float, so a round-trip never loses a digit.

fn json_write(v: &Value, out: &mut String, line: usize) -> RResult<()> {
    match v {
        Value::Nil => out.push_str("null"),
        Value::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
        Value::Int(b) => out.push_str(&b.to_string_signed()),
        Value::Dec(d) => out.push_str(&d.to_plain_string()),
        Value::Flt(f) => out.push_str(&crate::interp::fmt_f64(*f)),
        Value::Str(s) => json_write_str(s, out),
        Value::List(l) => {
            out.push('[');
            for (i, x) in l.borrow().iter().enumerate() {
                if i > 0 {
                    out.push_str(", ");
                }
                json_write(x, out, line)?;
            }
            out.push(']');
        }
        Value::Map(m) => {
            out.push('{');
            let b = m.borrow();
            for (i, k) in b.order.iter().enumerate() {
                if i > 0 {
                    out.push_str(", ");
                }
                match k {
                    crate::value::Key::Str(s) => json_write_str(s, out),
                    crate::value::Key::Int(n) => json_write_str(n, out),
                }
                out.push_str(": ");
                match b.get(k) {
                    Some(x) => json_write(x, out, line)?,
                    None => out.push_str("null"),
                }
            }
            out.push('}');
        }
        other => return Err(err(line,
            &format!("जेसन {} न पाठयति", other.type_name()),
            &format!("जेसन cannot serialise {}", other.type_name()))),
    }
    Ok(())
}

fn json_write_str(s: &str, out: &mut String) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            '\r' => out.push_str("\\r"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),   // ensure_ascii=False: real Unicode passes through
        }
    }
    out.push('"');
}

struct Json {
    c: Vec<char>,
    i: usize,
    line: usize,
}

impl Json {
    fn bad(&self, what: &str) -> String {
        err(self.line,
            &format!("जेसन-दोषः — {}", what),
            &format!("JSON error — {}", what))
    }
    fn ws(&mut self) {
        while self.i < self.c.len() && self.c[self.i].is_whitespace() {
            self.i += 1;
        }
    }
    fn peek(&self) -> Option<char> {
        self.c.get(self.i).copied()
    }
    fn lit(&mut self, word: &str) -> bool {
        let w: Vec<char> = word.chars().collect();
        if self.c.len() >= self.i + w.len() && self.c[self.i..self.i + w.len()] == w[..] {
            self.i += w.len();
            return true;
        }
        false
    }
    fn value(&mut self) -> RResult<Value> {
        self.ws();
        let c = match self.peek() {
            None => return Err(self.bad("अपूर्णम् / unexpected end")),
            Some(c) => c,
        };
        match c {
            'n' => if self.lit("null") { Ok(Value::Nil) }
                   else { Err(self.bad("'null' अपेक्षितम् / expected null")) },
            't' => if self.lit("true") { Ok(Value::Bool(true)) }
                   else { Err(self.bad("'true' अपेक्षितम् / expected true")) },
            'f' => if self.lit("false") { Ok(Value::Bool(false)) }
                   else { Err(self.bad("'false' अपेक्षितम् / expected false")) },
            '"' => self.string().map(Value::Str),
            '[' => {
                self.i += 1;
                let mut out = Vec::new();
                self.ws();
                if self.peek() == Some(']') { self.i += 1; return Ok(Value::list(out)); }
                loop {
                    out.push(self.value()?);
                    self.ws();
                    match self.peek() {
                        Some(',') => { self.i += 1; }
                        Some(']') => { self.i += 1; break; }
                        _ => return Err(self.bad("',' वा ']' अपेक्षितम् / expected ',' or ']'")),
                    }
                }
                Ok(Value::list(out))
            }
            '{' => {
                self.i += 1;
                let mut m = crate::value::MapData::default();
                self.ws();
                if self.peek() == Some('}') { self.i += 1; return Ok(Value::map(m)); }
                loop {
                    self.ws();
                    let k = self.string()?;
                    self.ws();
                    if self.peek() != Some(':') {
                        return Err(self.bad("':' अपेक्षितम् / expected ':'"));
                    }
                    self.i += 1;
                    let v = self.value()?;
                    m.insert(crate::value::Key::Str(k), v);
                    self.ws();
                    match self.peek() {
                        Some(',') => { self.i += 1; }
                        Some('}') => { self.i += 1; break; }
                        _ => return Err(self.bad("',' वा '}' अपेक्षितम् / expected ',' or '}'")),
                    }
                }
                Ok(Value::map(m))
            }
            _ => self.number(),
        }
    }
    fn string(&mut self) -> RResult<String> {
        if self.peek() != Some('"') {
            return Err(self.bad("वाक्यम् अपेक्षितम् / expected a string"));
        }
        self.i += 1;
        let mut out = String::new();
        while let Some(ch) = self.peek() {
            self.i += 1;
            match ch {
                '"' => return Ok(out),
                '\\' => {
                    let e = self.peek().ok_or_else(|| self.bad("अपूर्णम् / unexpected end"))?;
                    self.i += 1;
                    match e {
                        'n' => out.push('\n'),
                        't' => out.push('\t'),
                        'r' => out.push('\r'),
                        'b' => out.push('\u{0008}'),
                        'f' => out.push('\u{000C}'),
                        'u' => {
                            let hex: String = self.c.get(self.i..self.i + 4)
                                .ok_or_else(|| self.bad("\\u अपूर्णम् / bad \\u escape"))?
                                .iter().collect();
                            self.i += 4;
                            let n = u32::from_str_radix(&hex, 16)
                                .map_err(|_| self.bad("\\u अशुद्धम् / bad \\u escape"))?;
                            out.push(char::from_u32(n).unwrap_or('\u{FFFD}'));
                        }
                        other => out.push(other),
                    }
                }
                c => out.push(c),
            }
        }
        Err(self.bad("अपूर्णं वाक्यम् / unterminated string"))
    }
    fn number(&mut self) -> RResult<Value> {
        let start = self.i;
        if self.peek() == Some('-') { self.i += 1; }
        let mut fractional = false;
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() {
                self.i += 1;
            } else if c == '.' || c == 'e' || c == 'E' || c == '+' || c == '-' {
                fractional = true;
                self.i += 1;
            } else {
                break;
            }
        }
        let text: String = self.c[start..self.i].iter().collect();
        if text.is_empty() {
            return Err(self.bad("सङ्ख्या अपेक्षिता / expected a number"));
        }
        if fractional {
            // JSON allows exponents; the reference parses them with
            // parse_float=Decimal, which stays exact. Expand the exponent by
            // shifting the point ourselves rather than going through a float.
            let plain = expand_exponent(&text)
                .ok_or_else(|| self.bad(&format!("'{}' सङ्ख्या न / not a number", text)))?;
            Decimal::parse(&plain).map(Value::Dec)
                .ok_or_else(|| self.bad(&format!("'{}' सङ्ख्या न / not a number", text)))
        } else {
            let neg = text.starts_with('-');
            let body = text.strip_prefix('-').unwrap_or(&text);
            let mut b = BigInt::from_digits(body);
            if neg { b = b.neg(); }
            Ok(Value::Int(b))
        }
    }
}

/// "1e5" → "100000", "1.5e-3" → "0.0015". Returns None if the text is not a
/// well-formed number. No float is ever created.
fn expand_exponent(text: &str) -> Option<String> {
    let lower = text.to_ascii_lowercase();
    let (mant, exp) = match lower.split_once('e') {
        None => return Some(text.to_string()),
        Some((m, e)) => (m.to_string(), e.parse::<i32>().ok()?),
    };
    let neg = mant.starts_with('-');
    let body = mant.trim_start_matches(['-', '+']);
    let (int_part, frac_part) = match body.split_once('.') {
        Some((i, f)) => (i.to_string(), f.to_string()),
        None => (body.to_string(), String::new()),
    };
    if int_part.is_empty() && frac_part.is_empty() {
        return None;
    }
    if !int_part.chars().all(|c| c.is_ascii_digit())
        || !frac_part.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    let digits = format!("{}{}", int_part, frac_part);
    // position of the point, counted from the left, after applying the exponent
    let point = int_part.len() as i32 + exp;
    let mut out = String::new();
    if point <= 0 {
        out.push_str("0.");
        for _ in 0..(-point) { out.push('0'); }
        out.push_str(&digits);
    } else if point as usize >= digits.len() {
        out.push_str(&digits);
        for _ in 0..(point as usize - digits.len()) { out.push('0'); }
    } else {
        out.push_str(&digits[..point as usize]);
        out.push('.');
        out.push_str(&digits[point as usize..]);
    }
    Some(if neg { format!("-{}", out) } else { out })
}

fn json_parse(text: &str, line: usize) -> RResult<Value> {
    let mut p = Json { c: text.chars().collect(), i: 0, line };
    let v = p.value()?;
    p.ws();
    if p.i < p.c.len() {
        return Err(p.bad("अतिरिक्ताः अक्षराः / trailing characters"));
    }
    Ok(v)
}

#[cfg(test)]
mod json_tests {
    use super::*;

    #[test]
    fn exponents_expand_without_a_float() {
        assert_eq!(expand_exponent("1e5").unwrap(), "100000");
        assert_eq!(expand_exponent("1.5e-3").unwrap(), "0.0015");
        assert_eq!(expand_exponent("-2.5e2").unwrap(), "-250");
        assert_eq!(expand_exponent("0.1").unwrap(), "0.1");
        assert_eq!(expand_exponent("12").unwrap(), "12");
    }

    #[test]
    fn json_round_trip_keeps_the_scale() {
        let v = json_parse("{\"a\": 0.10, \"b\": [1, 2.5], \"c\": null}", 1).unwrap();
        let mut out = String::new();
        json_write(&v, &mut out, 1).unwrap();
        assert_eq!(out, "{\"a\": 0.10, \"b\": [1, 2.5], \"c\": null}");
    }

    #[test]
    fn malformed_json_is_an_error() {
        assert!(json_parse("{अ}", 1).is_err());
        assert!(json_parse("[1, 2", 1).is_err());
        assert!(json_parse("[1] junk", 1).is_err());
    }
}

// ---- tiny helpers (no external crates) ----

fn unix_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn next_rand() -> u64 {
    use std::cell::Cell;
    thread_local! {
        static STATE: Cell<u64> = const { Cell::new(0) };
    }
    STATE.with(|s| {
        let mut x = s.get();
        if x == 0 {
            x = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos() as u64)
                .unwrap_or(0x2545F4914F6CDD1D)
                | 1;
        }
        // xorshift64*
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        s.set(x);
        x.wrapping_mul(0x2545F4914F6CDD1D) >> 1
    })
}

/// Civil date from a Unix timestamp (Howard Hinnant's algorithm).
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

fn today() -> String {
    let (y, m, d) = civil_from_days((unix_secs() / 86_400) as i64);
    format!("{:04}-{:02}-{:02}", y, m, d)
}

fn year() -> i64 {
    civil_from_days((unix_secs() / 86_400) as i64).0
}

fn clock() -> String {
    let s = unix_secs() % 86_400;
    format!("{:02}:{:02}:{:02}", s / 3600, (s % 3600) / 60, s % 60)
}
