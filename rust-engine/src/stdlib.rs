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
                           "ज्या", "कोज्या", "पाई", "ई", "परिवृत्त"];
const VAKYAKARMA: &[&str] = &["विभज", "संयोजय", "खोज", "प्रतिस्थापय", "अंश",
                              "उच्च", "निम्न", "परिष्कार", "आरभते", "अन्तयति",
                              "अन्तर्भवति", "आकारय", "पूरय"];
const YADRCCHIKAM: &[&str] = &["अन्तरे", "वरय", "भिन्नम्"];
const KALAH: &[&str] = &["अद्य", "संप्रति", "कालमुद्रा", "वर्षः", "मासः",
                         "दिनम्", "वासरः", "दिनयोगः", "अन्तरम्", "पूर्वम्",
                         "शुद्धः", "रूपय", "अधिवर्षः", "क्षणविरामः"];
const NIYAMITAM: &[&str] = &["मेलति", "आदिमेलति", "खोज", "सर्वाणि", "स्थानम्",
                             "प्रतिस्थापय", "विभज", "समूहाः"];
const SARANI: &[&str] = &["विश्लेषय", "कोशाः", "पाठय"];
const GUDHA: &[&str] = &["सङ्क्षेपः", "एकाकी", "गूढय", "प्रकटय"];
const PARIVESHA: &[&str] = &["चरः", "चराः", "निर्गम", "दोषवद",
                             "कार्यसूचिका", "मञ्चः"];
const LEKHANI: &[&str] = &["विवरणम्", "सूचना", "चेतावनी", "दोषः", "महादोषः",
                           "स्तरः", "सञ्चिकायाम्"];
const SUCHIKARMA: &[&str] = &["छानय", "प्रतिचित्रय", "न्यूनीकरण", "विपर्यय",
                              "अन्तर्भवति", "अनुक्रमः", "योगः", "महत्तमम्",
                              "लघुत्तमम्", "अद्वितीयम्", "क्रमय", "सङ्गमः",
                              "सम्पातः", "भेदः"];
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
        "नियमितम्" => NIYAMITAM,
        "सारणी" => SARANI,
        "गूढ" => GUDHA,
        "परिवेशः" => PARIVESHA,
        "लेखनी" => LEKHANI,
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
const LIST_BUILTINS: &[&str] = &["दैर्घ्यम्", "योजय", "अपनय"];
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
    errk(line, sa, en, "दोषः")
}

/// Same, but naming the kind so a program can branch on it.
fn errk(line: usize, sa: &str, en: &str, kind: &str) -> String {
    crate::err::SError::new(line, sa, en, kind).encode()
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

/// Ordering for क्रमय: numbers among themselves, then text among itself.
fn sort_cmp(it: &Interp, a: &Value, b: &Value, line: usize)
    -> RResult<std::cmp::Ordering>
{
    use std::cmp::Ordering;
    let rank = |v: &Value| if v.is_numeric() && !matches!(v, Value::Bool(_)) { 0 } else { 1 };
    match rank(a).cmp(&rank(b)) {
        Ordering::Equal => {}
        other => return Ok(other),
    }
    if rank(a) == 0 {
        if it.greater_than(a, b, line)? {
            return Ok(Ordering::Greater);
        }
        if it.greater_than(b, a, line)? {
            return Ok(Ordering::Less);
        }
        return Ok(Ordering::Equal);
    }
    Ok(it.display(a).cmp(&it.display(b)))
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
        ("गणितम्", "परिवृत्त") => {
            // the explicit "make it money" step — see Decimal::round_half_up
            let places = match vals.get(1) {
                None | Some(Value::Nil) => 0,
                Some(v) => {
                    let n = want_int(Some(v), line, "परिवृत्त()")?;
                    if n < 0 {
                        return Err(err(line, "स्थानानि ऋणात्मकानि न भवेयुः",
                                       "places cannot be negative"));
                    }
                    n as usize
                }
            };
            match vals.first() {
                Some(v) if v.is_number() => {
                    let d = v.as_decimal().ok_or_else(|| err(line,
                        "परिवृत्त() शुद्धां सङ्ख्याम् अपेक्षते",
                        "परिवृत्त() expects an exact number"))?;
                    Ok(Value::Dec(d.round_half_up(places)))
                }
                _ => Err(err(line,
                    "परिवृत्त() शुद्धां सङ्ख्याम् अपेक्षते (पूर्णाङ्कः वा दशमांशः)",
                    "परिवृत्त() expects an exact number (पूर्णाङ्कः or दशमांशः)")),
            }
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
                return Err(errk(line, "अन्तरे(आदिः, अन्तः) — अन्तः न्यूनः",
                                "अन्तरे(low, high) — high is less than low",
                                "सीमादोषः"));
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

        ("सूचीकर्म", "क्रमय") => {
            // क्रमय(सूची) sorts; क्रमय(सूची, विधिः) sorts by what विधिः returns.
            // Stable, and numbers sort before text — mixing types in one सूची is
            // unusual, and this beats refusing to sort at all.
            let items = want_list(vals.first(), line, "क्रमय()")?;
            let keyfn = vals.get(1).cloned();
            let mut keyed: Vec<(Value, Value)> = Vec::with_capacity(items.len());
            for v in items {
                let k = match &keyfn {
                    Some(f) => it.apply(f, vec![v.clone()], line)?,
                    None => v.clone(),
                };
                keyed.push((k, v));
            }
            let mut err: Option<String> = None;
            keyed.sort_by(|a, b| {
                if err.is_some() {
                    return std::cmp::Ordering::Equal;
                }
                match sort_cmp(it, &a.0, &b.0, line) {
                    Ok(o) => o,
                    Err(e) => { err = Some(e); std::cmp::Ordering::Equal }
                }
            });
            if let Some(e) = err {
                return Err(e);
            }
            Ok(Value::list(keyed.into_iter().map(|(_, v)| v).collect()))
        }
        ("सूचीकर्म", "सङ्गमः") => {
            let a = want_list(vals.first(), line, "सङ्गमः()")?;
            let b = want_list(vals.get(1), line, "सङ्गमः()")?;
            let mut out: Vec<Value> = Vec::new();
            for v in a.into_iter().chain(b) {
                if !out.iter().any(|x| *x == v) {
                    out.push(v);
                }
            }
            Ok(Value::list(out))
        }
        ("सूचीकर्म", "सम्पातः") => {
            let a = want_list(vals.first(), line, "सम्पातः()")?;
            let b = want_list(vals.get(1), line, "सम्पातः()")?;
            let mut out: Vec<Value> = Vec::new();
            for v in a {
                if b.iter().any(|x| *x == v) && !out.iter().any(|x| *x == v) {
                    out.push(v);
                }
            }
            Ok(Value::list(out))
        }
        ("सूचीकर्म", "भेदः") => {
            let a = want_list(vals.first(), line, "भेदः()")?;
            let b = want_list(vals.get(1), line, "भेदः()")?;
            Ok(Value::list(a.into_iter()
                .filter(|v| !b.iter().any(|x| x == v)).collect()))
        }

        // ---------------- वाक्यकर्म.आकारय ----------------
        ("वाक्यकर्म", "आकारय") => {
            let pat = want_str(vals.first(), line, "आकारय()")?;
            let args = &vals[1.min(vals.len())..];
            let parts: Vec<&str> = pat.split("{}").collect();
            if parts.len() - 1 != args.len() {
                return Err(err(line,
                    &format!("आकारय(): {} स्थानानि, {} मूल्यानि",
                             dev_digits(&(parts.len() - 1).to_string()),
                             dev_digits(&args.len().to_string())),
                    &format!("आकारय(): {} placeholders but {} values",
                             parts.len() - 1, args.len())));
            }
            let mut out = String::from(parts[0]);
            for (v, tail) in args.iter().zip(parts[1..].iter()) {
                out.push_str(&it.display(v));
                out.push_str(tail);
            }
            Ok(Value::Str(out))
        }

        ("वाक्यकर्म", "पूरय") => {
            // Padding counts CHARACTERS, not display columns: a Devanagari
            // conjunct may render narrower than a Latin letter, so a padded
            // column is approximately aligned. Said plainly rather than
            // pretended otherwise.
            let text = match vals.first() {
                Some(Value::Str(s)) => s.clone(),
                Some(other) => it.display(other),
                None => return Err(err(line, "पूरय(पाठः, विस्तारः)",
                                       "पूरय(text, width)")),
            };
            let width = want_int(vals.get(1), line, "पूरय()")?;
            let pad = width.abs() - text.chars().count() as i64;
            if pad <= 0 {
                return Ok(Value::Str(text));
            }
            let spaces: String = " ".repeat(pad as usize);
            Ok(Value::Str(if width < 0 { format!("{}{}", spaces, text) }
                          else { format!("{}{}", text, spaces) }))
        }

        // ---------------- नियमितम् ----------------
        // Regular expressions live only in the reference engine for now. वेगः
        // has no external crates by design, and a hand-written engine that is
        // not byte-identical to the reference would be worse than none: it
        // would make the same pattern mean two things. Saying so plainly is the
        // honest option until the subset engine lands.
        ("नियमितम्", _) => Err(err(line,
            "नियमितम् वेगे नास्ति — मूल-इञ्जिने (sanskrita.py) प्रयुज्यताम्",
            "नियमितम् (regex) is not in the वेगः engine yet — run this program \
             with the reference engine (sanskrita.py). Tracked in AUDIT.md.")),

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

        // ---------------- सारणी (CSV, RFC 4180) ----------------
        // Every field comes back as वाक्यम्. Numbers are never guessed at —
        // converting is your decision, made with सङ्ख्या(), not a silent one.
        ("सारणी", "विश्लेषय") => {
            let text = want_str(vals.first(), line, "विश्लेषय()")?;
            let sep = csv_sep(vals.get(1), line)?;
            Ok(Value::list(csv_parse(&text, sep).into_iter()
                .map(|row| Value::list(row.into_iter().map(Value::Str).collect()))
                .collect()))
        }
        ("सारणी", "कोशाः") => {
            let text = want_str(vals.first(), line, "कोशाः()")?;
            let sep = csv_sep(vals.get(1), line)?;
            let rows = csv_parse(&text, sep);
            if rows.is_empty() {
                return Ok(Value::list(Vec::new()));
            }
            let head = &rows[0];
            let mut out = Vec::new();
            for row in &rows[1..] {
                let mut m = crate::value::MapData::default();
                for (i, cell) in row.iter().enumerate() {
                    let key = head.get(i).cloned()
                        .unwrap_or_else(|| (i + 1).to_string());
                    m.insert(crate::value::Key::Str(key), Value::Str(cell.clone()));
                }
                out.push(Value::map(m));
            }
            Ok(Value::list(out))
        }
        ("सारणी", "पाठय") => {
            let rows = want_list(vals.first(), line, "पाठय()")?;
            let sep = csv_sep(vals.get(1), line)?;
            let mut out = String::new();
            let map_style = matches!(rows.first(), Some(Value::Map(_)));
            let mut head: Vec<crate::value::Key> = Vec::new();
            if map_style {
                if let Some(Value::Map(m)) = rows.first() {
                    head = m.borrow().order.clone();
                }
                let names: Vec<String> = head.iter().map(|k| match k {
                    crate::value::Key::Str(s) => s.clone(),
                    crate::value::Key::Int(n) => dev_digits(n),
                }).collect();
                csv_row(&names, sep, &mut out);
            }
            for r in &rows {
                match r {
                    Value::Map(m) if map_style => {
                        let b = m.borrow();
                        let cells: Vec<String> = head.iter()
                            .map(|k| b.get(k).map(|v| csv_cell(it, v))
                                 .unwrap_or_default()).collect();
                        csv_row(&cells, sep, &mut out);
                    }
                    Value::List(l) => {
                        let cells: Vec<String> = l.borrow().iter()
                            .map(|v| csv_cell(it, v)).collect();
                        csv_row(&cells, sep, &mut out);
                    }
                    _ => return Err(err(line,
                        "पाठय() प्रतिपङ्क्ति सूचीं कोशं वा अपेक्षते",
                        "पाठय() expects each row to be a सूची or a कोशः")),
                }
            }
            Ok(Value::Str(out))
        }

        // ---------------- गूढ ----------------
        // Note what is deliberately absent: encryption. A language this young
        // has no business holding anyone's secrets.
        ("गूढ", "सङ्क्षेपः") => {
            let text = want_str(vals.first(), line, "सङ्क्षेपः()")?;
            let algo = match vals.get(1) {
                Some(Value::Str(s)) => ascii_digits(s).to_lowercase(),
                None => "sha256".to_string(),
                _ => return Err(err(line, "सङ्क्षेपः() विधिं वाक्यरूपेण अपेक्षते",
                                    "सङ्क्षेपः() expects the algorithm as text")),
            };
            match algo.as_str() {
                "sha256" => Ok(Value::Str(hex(&sha256(text.as_bytes())))),
                other => Err(err(line,
                    &format!("'{}' इति विधिः वेगे नास्ति — sha256 एव", other),
                    &format!("'{}' is not available in वेगः — only sha256 is \
                              implemented here (the reference engine has \
                              sha512, sha1 and md5 too)", other))),
            }
        }
        ("गूढ", "एकाकी") => Ok(Value::Str(uuid4())),
        ("गूढ", "गूढय") => {
            let t = want_str(vals.first(), line, "गूढय()")?;
            Ok(Value::Str(b64_encode(t.as_bytes())))
        }
        ("गूढ", "प्रकटय") => {
            let t = want_str(vals.first(), line, "प्रकटय()")?;
            let bytes = b64_decode(&t).ok_or_else(|| err(line,
                "अशुद्धं base64", "not valid base64"))?;
            String::from_utf8(bytes).map(Value::Str).map_err(|_| err(line,
                "base64 अन्तर्गतं UTF-8 न", "base64 did not contain UTF-8 text"))
        }

        // ---------------- परिवेशः ----------------
        ("परिवेशः", "चरः") => {
            let name_ = want_str(vals.first(), line, "चरः()")?;
            match std::env::var(&name_) {
                Ok(v) => Ok(Value::Str(v)),
                Err(_) => Ok(vals.get(1).cloned().unwrap_or(Value::Nil)),
            }
        }
        ("परिवेशः", "चराः") => {
            let mut m = crate::value::MapData::default();
            let mut pairs: Vec<(String, String)> = std::env::vars().collect();
            pairs.sort();                       // deterministic, so tests can rely on it
            for (k, v) in pairs {
                m.insert(crate::value::Key::Str(k), Value::Str(v));
            }
            Ok(Value::map(m))
        }
        ("परिवेशः", "निर्गम") => {
            let code = match vals.first() {
                None | Some(Value::Nil) => 0,
                Some(v) => want_int(Some(v), line, "निर्गम()")?,
            };
            use std::io::Write;
            let _ = std::io::stdout().flush();
            std::process::exit(code as i32);
        }
        ("परिवेशः", "दोषवद") => {
            let parts: Vec<String> = vals.iter().map(|v| it.display(v)).collect();
            eprintln!("{}", parts.join(" "));
            Ok(Value::Nil)
        }
        ("परिवेशः", "कार्यसूचिका") => Ok(Value::Str(
            std::env::current_dir().map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default())),
        ("परिवेशः", "मञ्चः") => Ok(Value::Str(
            if cfg!(target_os = "macos") { "Darwin" }
            else if cfg!(target_os = "windows") { "Windows" }
            else if cfg!(target_os = "linux") { "Linux" }
            else { "अज्ञातः" }.to_string())),

        // ---------------- लेखनी ----------------
        // Logs go to stderr so they never contaminate a program's real output.
        ("लेखनी", "स्तरः") => {
            let want = want_str(vals.first(), line, "स्तरः()")?;
            match log_rank(&want) {
                Some(r) => { LOG_LEVEL.with(|c| c.set(r)); Ok(Value::Nil) }
                None => Err(err(line,
                    &format!("'{}' इति स्तरः न ज्ञातः", want),
                    &format!("unknown level '{}' — विवरणम्, सूचना, चेतावनी, \
                              दोषः, महादोषः", want))),
            }
        }
        ("लेखनी", "सञ्चिकायाम्") => {
            let path = match vals.first() {
                Some(Value::Str(s)) => Some(s.clone()),
                None | Some(Value::Nil) => None,
                _ => return Err(err(line, "सञ्चिकायाम्() मार्गं वाक्यरूपेण अपेक्षते",
                                    "सञ्चिकायाम्() expects a path as text")),
            };
            LOG_FILE.with(|c| *c.borrow_mut() = path);
            Ok(Value::Nil)
        }
        ("लेखनी", _) => {
            let rank = log_rank(name).ok_or_else(|| err(line,
                &format!("'{}' कोष्ठके 'लेखनी' नास्ति", name),
                &format!("module 'लेखनी' has no '{}'", name)))?;
            if rank < LOG_LEVEL.with(|c| c.get()) {
                return Ok(Value::Nil);
            }
            let text: Vec<String> = vals.iter().map(|v| it.display(v)).collect();
            let (y, m, d) = civil_from_days((unix_secs() / 86_400) as i64);
            let stamp = dev_digits(&format!("{:04}-{:02}-{:02} {}", y, m, d, clock()));
            let entry = format!("[{}] {}: {}", stamp, name, text.join(" "));
            let path = LOG_FILE.with(|c| c.borrow().clone());
            match path {
                Some(p) => {
                    use std::io::Write;
                    if let Ok(mut f) = std::fs::OpenOptions::new()
                        .create(true).append(true).open(&p) {
                        let _ = writeln!(f, "{}", entry);
                    }
                }
                None => eprintln!("{}", entry),
            }
            Ok(Value::Nil)
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
        // A date is ISO text ("२०२६-०७-२५"), because text is what files, JSON
        // and users hand you. Devanagari or ASCII digits both parse.
        ("कालः", "अद्य") => Ok(Value::Str(dev_digits(&today()))),
        ("कालः", "संप्रति") => Ok(Value::Str(dev_digits(&clock()))),
        ("कालः", "कालमुद्रा") => Ok(Value::int(unix_secs() as i64)),
        ("कालः", "वर्षः") | ("कालः", "मासः") | ("कालः", "दिनम्") => {
            let (y, m, d) = match vals.first() {
                None | Some(Value::Nil) => civil_from_days((unix_secs() / 86_400) as i64),
                Some(v) => parse_date(v, line, name)?,
            };
            Ok(Value::int(match name {
                "वर्षः" => y,
                "मासः" => m as i64,
                _ => d as i64,
            }))
        }
        ("कालः", "वासरः") => {
            const VARA: [&str; 7] = ["सोमवासरः", "मङ्गलवासरः", "बुधवासरः",
                                     "गुरुवासरः", "शुक्रवासरः", "शनिवासरः",
                                     "रविवासरः"];
            let days = match vals.first() {
                None | Some(Value::Nil) => (unix_secs() / 86_400) as i64,
                Some(v) => {
                    let (y, m, d) = parse_date(v, line, "वासरः()")?;
                    days_from_civil(y, m, d)
                }
            };
            // 1970-01-01 was a Thursday → index 3 in a Monday-first week
            let idx = (((days % 7) + 7 + 3) % 7) as usize;
            Ok(Value::Str(VARA[idx].to_string()))
        }
        ("कालः", "दिनयोगः") => {
            let (y, m, d) = parse_date(vals.first().unwrap_or(&Value::Nil),
                                       line, "दिनयोगः()")?;
            let n = want_int(vals.get(1), line, "दिनयोगः()")?;
            let (y2, m2, d2) = civil_from_days(days_from_civil(y, m, d) + n);
            Ok(Value::Str(dev_digits(&format!("{:04}-{:02}-{:02}", y2, m2, d2))))
        }
        ("कालः", "अन्तरम्") => {
            let (ya, ma, da) = parse_date(vals.first().unwrap_or(&Value::Nil),
                                          line, "अन्तरम्()")?;
            let (yb, mb, db) = parse_date(vals.get(1).unwrap_or(&Value::Nil),
                                          line, "अन्तरम्()")?;
            Ok(Value::int(days_from_civil(yb, mb, db) - days_from_civil(ya, ma, da)))
        }
        ("कालः", "पूर्वम्") => {
            let (ya, ma, da) = parse_date(vals.first().unwrap_or(&Value::Nil),
                                          line, "पूर्वम्()")?;
            let (yb, mb, db) = parse_date(vals.get(1).unwrap_or(&Value::Nil),
                                          line, "पूर्वम्()")?;
            Ok(Value::Bool(days_from_civil(ya, ma, da) < days_from_civil(yb, mb, db)))
        }
        ("कालः", "शुद्धः") => Ok(Value::Bool(
            parse_date(vals.first().unwrap_or(&Value::Nil), line, "शुद्धः()").is_ok())),
        ("कालः", "रूपय") => {
            let (y, m, d) = parse_date(vals.first().unwrap_or(&Value::Nil),
                                       line, "रूपय()")?;
            let pat = match vals.get(1) {
                Some(Value::Str(s)) => ascii_digits(s),
                None => "%Y-%m-%d".to_string(),
                _ => return Err(err(line, "रूपय() आकारं वाक्यरूपेण अपेक्षते",
                                    "रूपय() expects the pattern as text")),
            };
            Ok(Value::Str(dev_digits(&strftime(&pat, y, m, d))))
        }
        ("कालः", "अधिवर्षः") => {
            let y = match vals.first() {
                None | Some(Value::Nil) => civil_from_days((unix_secs() / 86_400) as i64).0,
                Some(v) => want_int(Some(v), line, "अधिवर्षः()")?,
            };
            Ok(Value::Bool(y % 4 == 0 && (y % 100 != 0 || y % 400 == 0)))
        }
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

// ---- सारणी: CSV, hand-written to RFC 4180 (no crates) ----

fn csv_sep(v: Option<&Value>, line: usize) -> RResult<char> {
    match v {
        None | Some(Value::Nil) => Ok(','),
        Some(Value::Str(s)) => s.chars().next().ok_or_else(|| err(line,
            "विभाजकः रिक्तः न भवेत्", "the delimiter cannot be empty")),
        _ => Err(err(line, "विभाजकः वाक्यम् भवेत्",
                     "the delimiter must be text")),
    }
}

fn csv_parse(text: &str, sep: char) -> Vec<Vec<String>> {
    let mut rows = Vec::new();
    let mut row: Vec<String> = Vec::new();
    let mut field = String::new();
    let mut in_quotes = false;
    let mut any = false;
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if in_quotes {
            if c == '"' {
                if chars.get(i + 1) == Some(&'"') {   // "" is a literal quote
                    field.push('"');
                    i += 2;
                    continue;
                }
                in_quotes = false;
            } else {
                field.push(c);
            }
            i += 1;
            continue;
        }
        match c {
            '"' => { in_quotes = true; any = true; }
            _ if c == sep => { row.push(std::mem::take(&mut field)); any = true; }
            '\r' => {}
            '\n' => {
                row.push(std::mem::take(&mut field));
                rows.push(std::mem::take(&mut row));
                any = false;
            }
            _ => { field.push(c); any = true; }
        }
        i += 1;
    }
    if any || !field.is_empty() || !row.is_empty() {
        row.push(field);
        rows.push(row);
    }
    rows
}

fn csv_cell(it: &Interp, v: &Value) -> String {
    match v {
        Value::Nil => String::new(),
        Value::Str(s) => s.clone(),
        other => it.display(other),      // दशमांशः keeps its exact digits here
    }
}

fn csv_row(cells: &[String], sep: char, out: &mut String) {
    for (i, c) in cells.iter().enumerate() {
        if i > 0 {
            out.push(sep);
        }
        if c.contains(sep) || c.contains('"') || c.contains('\n') || c.contains('\r') {
            out.push('"');
            for ch in c.chars() {
                if ch == '"' { out.push('"'); }
                out.push(ch);
            }
            out.push('"');
        } else {
            out.push_str(c);
        }
    }
    out.push('\n');
}

// ---- गूढ: SHA-256, base64 and UUID4, hand-written (no crates) ----

const K256: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1,
    0x923f82a4, 0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3,
    0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786,
    0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147,
    0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13,
    0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
    0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a,
    0x5b9cca4f, 0x682e6ff3, 0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208,
    0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

fn sha256(data: &[u8]) -> [u8; 32] {
    let mut h: [u32; 8] = [0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a,
                           0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19];
    let mut msg = data.to_vec();
    let bitlen = (data.len() as u64) * 8;
    msg.push(0x80);
    while msg.len() % 64 != 56 {
        msg.push(0);
    }
    msg.extend_from_slice(&bitlen.to_be_bytes());

    for block in msg.chunks(64) {
        let mut w = [0u32; 64];
        for i in 0..16 {
            w[i] = u32::from_be_bytes([block[4 * i], block[4 * i + 1],
                                       block[4 * i + 2], block[4 * i + 3]]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18)
                     ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19)
                     ^ (w[i - 2] >> 10);
            w[i] = w[i - 16].wrapping_add(s0).wrapping_add(w[i - 7]).wrapping_add(s1);
        }
        let (mut a, mut b, mut c, mut d) = (h[0], h[1], h[2], h[3]);
        let (mut e, mut f, mut g, mut hh) = (h[4], h[5], h[6], h[7]);
        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let t1 = hh.wrapping_add(s1).wrapping_add(ch)
                       .wrapping_add(K256[i]).wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let t2 = s0.wrapping_add(maj);
            hh = g; g = f; f = e;
            e = d.wrapping_add(t1);
            d = c; c = b; b = a;
            a = t1.wrapping_add(t2);
        }
        for (i, v) in [a, b, c, d, e, f, g, hh].iter().enumerate() {
            h[i] = h[i].wrapping_add(*v);
        }
    }
    let mut out = [0u8; 32];
    for (i, v) in h.iter().enumerate() {
        out[4 * i..4 * i + 4].copy_from_slice(&v.to_be_bytes());
    }
    out
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

const B64: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

fn b64_encode(data: &[u8]) -> String {
    let mut out = String::new();
    for chunk in data.chunks(3) {
        let b = [chunk[0], *chunk.get(1).unwrap_or(&0), *chunk.get(2).unwrap_or(&0)];
        let n = ((b[0] as u32) << 16) | ((b[1] as u32) << 8) | b[2] as u32;
        out.push(B64[(n >> 18) as usize & 63] as char);
        out.push(B64[(n >> 12) as usize & 63] as char);
        out.push(if chunk.len() > 1 { B64[(n >> 6) as usize & 63] as char } else { '=' });
        out.push(if chunk.len() > 2 { B64[n as usize & 63] as char } else { '=' });
    }
    out
}

fn b64_decode(text: &str) -> Option<Vec<u8>> {
    let mut bits: Vec<u8> = Vec::new();
    for c in text.chars() {
        if c == '=' || c.is_whitespace() {
            continue;
        }
        bits.push(B64.iter().position(|b| *b as char == c)? as u8);
    }
    let mut out = Vec::new();
    for chunk in bits.chunks(4) {
        let mut n: u32 = 0;
        for (i, v) in chunk.iter().enumerate() {
            n |= (*v as u32) << (18 - 6 * i);
        }
        let bytes = match chunk.len() { 2 => 1, 3 => 2, 4 => 3, _ => return None };
        for i in 0..bytes {
            out.push(((n >> (16 - 8 * i)) & 0xFF) as u8);
        }
    }
    Some(out)
}

/// A version-4 UUID from the same xorshift source as यादृच्छिकम्.
fn uuid4() -> String {
    let mut b = [0u8; 16];
    for chunk in b.chunks_mut(8) {
        let r = next_rand().to_le_bytes();
        chunk.copy_from_slice(&r[..chunk.len()]);
    }
    b[6] = (b[6] & 0x0F) | 0x40;      // version 4
    b[8] = (b[8] & 0x3F) | 0x80;      // variant 1
    let h = hex(&b);
    format!("{}-{}-{}-{}-{}", &h[0..8], &h[8..12], &h[12..16], &h[16..20], &h[20..32])
}

// ---- लेखनी: level and destination, per thread ----

thread_local! {
    static LOG_LEVEL: std::cell::Cell<u8> = const { std::cell::Cell::new(20) };
    static LOG_FILE: std::cell::RefCell<Option<String>> =
        const { std::cell::RefCell::new(None) };
}

fn log_rank(name: &str) -> Option<u8> {
    match name {
        "विवरणम्" => Some(10),
        "सूचना" => Some(20),
        "चेतावनी" => Some(30),
        "दोषः" => Some(40),
        "महादोषः" => Some(50),
        _ => None,
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

/// Devanagari digits → ASCII, so every कालः function takes either script.
fn ascii_digits(s: &str) -> String {
    s.chars().map(|c| match c {
        '०'..='९' => char::from(b'0' + (c as u32 - '०' as u32) as u8),
        other => other,
    }).collect()
}

/// Parse a date written as text. Four shapes are accepted, matching the
/// reference: Y-M-D, Y/M/D, D-M-Y, D/M/Y.
fn parse_date(v: &Value, line: usize, who: &str) -> RResult<(i64, u32, u32)> {
    let text = match v {
        Value::Str(s) => ascii_digits(s.trim()),
        _ => return Err(err(line,
            &format!("{} दिनाङ्कं वाक्यरूपेण अपेक्षते", who),
            &format!("{} expects a date as text, e.g. \"२०२६-०७-२५\"", who))),
    };
    let bad = || err(line,
        &format!("'{}' इति दिनाङ्कः न ज्ञातः", text),
        &format!("'{}' is not a date I recognise (try २०२६-०७-२५)", text));
    let sep = if text.contains('-') { '-' } else { '/' };
    let parts: Vec<&str> = text.split(sep).collect();
    if parts.len() != 3 {
        return Err(bad());
    }
    let n: Vec<i64> = parts.iter().map(|p| p.parse::<i64>().unwrap_or(-1)).collect();
    if n.iter().any(|x| *x < 0) {
        return Err(bad());
    }
    // four digits first means Y-M-D; otherwise D-M-Y
    let (y, m, d) = if parts[0].len() == 4 { (n[0], n[1], n[2]) }
                    else { (n[2], n[1], n[0]) };
    if !(1..=12).contains(&m) || !(1..=31).contains(&d) {
        return Err(bad());
    }
    // reject the impossible: 31 April, 30 February, 29 February in a common year
    let leap = y % 4 == 0 && (y % 100 != 0 || y % 400 == 0);
    let len = [31, if leap { 29 } else { 28 }, 31, 30, 31, 30,
               31, 31, 30, 31, 30, 31][(m - 1) as usize];
    if d > len {
        return Err(bad());
    }
    Ok((y, m as u32, d as u32))
}

/// Days since 1970-01-01 (Howard Hinnant's algorithm, the inverse of
/// civil_from_days).
fn days_from_civil(y: i64, m: u32, d: u32) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = (y - era * 400) as u64;
    let mp = if m > 2 { m - 3 } else { m + 9 } as u64;
    let doy = (153 * mp + 2) / 5 + (d as u64 - 1);
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe as i64 - 719_468
}

/// The strftime directives a date can use. Anything else is copied through.
fn strftime(pat: &str, y: i64, m: u32, d: u32) -> String {
    const MON: [&str; 12] = ["Jan", "Feb", "Mar", "Apr", "May", "Jun",
                             "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
    let mut out = String::new();
    let chars: Vec<char> = pat.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '%' && i + 1 < chars.len() {
            match chars[i + 1] {
                'Y' => out.push_str(&format!("{:04}", y)),
                'y' => out.push_str(&format!("{:02}", y % 100)),
                'm' => out.push_str(&format!("{:02}", m)),
                'd' => out.push_str(&format!("{:02}", d)),
                'b' => out.push_str(MON[(m - 1) as usize]),
                '%' => out.push('%'),
                other => { out.push('%'); out.push(other); }
            }
            i += 2;
            continue;
        }
        out.push(chars[i]);
        i += 1;
    }
    out
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

fn clock() -> String {
    let s = unix_secs() % 86_400;
    format!("{:02}:{:02}:{:02}", s / 3600, (s % 3600) / 60, s % 60)
}

#[cfg(test)]
mod tier2_tests {
    use super::*;

    // verified against Python's hashlib for the same inputs
    #[test]
    fn sha256_matches_the_reference() {
        assert_eq!(hex(&sha256(b"")),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
        assert_eq!(hex(&sha256("नमस्ते".as_bytes())),
            "ddb08d77c2d511947652161b35987022711aa216387b041dd459a03eb66a8304");
        assert_eq!(hex(&sha256(b"abc")),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
        // the block-boundary cases where padding usually goes wrong
        assert_eq!(hex(&sha256(&[b'a'; 55])),
            "9f4390f8d30c2dd92ec9f095b65e2b9ae9b0a925a5258e241c9f1e910f734318");
        assert_eq!(hex(&sha256(&[b'a'; 56])),
            "b35439a4ac6f0948b6d6f9e3c6af0f5f590ce20f1bde7090ef7970686ec6738a");
    }

    #[test]
    fn base64_round_trips() {
        assert_eq!(b64_encode("नमस्ते".as_bytes()), "4KSo4KSu4KS44KWN4KSk4KWH");
        assert_eq!(b64_encode(b"a"), "YQ==");
        assert_eq!(b64_encode(b"ab"), "YWI=");
        assert_eq!(b64_encode(b"abc"), "YWJj");
        for t in ["", "a", "ab", "abc", "नमस्ते जगत्"] {
            let e = b64_encode(t.as_bytes());
            assert_eq!(String::from_utf8(b64_decode(&e).unwrap()).unwrap(), t);
        }
        assert!(b64_decode("!!!").is_none());
    }

    #[test]
    fn uuid4_has_the_right_shape() {
        let u = uuid4();
        assert_eq!(u.len(), 36);
        let parts: Vec<&str> = u.split('-').collect();
        assert_eq!(parts.iter().map(|p| p.len()).collect::<Vec<_>>(),
                   vec![8, 4, 4, 4, 12]);
        assert!(u.as_bytes()[14] == b'4');                       // version
        assert!("89ab".contains(u.chars().nth(19).unwrap()));    // variant
        assert_ne!(uuid4(), uuid4());
    }

    #[test]
    fn csv_handles_quotes_commas_and_newlines() {
        let rows = csv_parse("नाम,नगरम्\n\"शर्मा, राम\",काशी\n", ',');
        assert_eq!(rows, vec![vec!["नाम", "नगरम्"], vec!["शर्मा, राम", "काशी"]]);
        // "" inside a quoted field is one literal quote
        assert_eq!(csv_parse("\"a\"\"b\"\n", ','), vec![vec!["a\"b"]]);
        // a newline inside quotes does not end the row
        assert_eq!(csv_parse("\"one\ntwo\",x\n", ','),
                   vec![vec!["one\ntwo", "x"]]);
    }

    #[test]
    fn csv_quotes_only_what_needs_it() {
        let mut out = String::new();
        csv_row(&["अ".into(), "ब, स".into(), "d\"e".into()], ',', &mut out);
        assert_eq!(out, "अ,\"ब, स\",\"d\"\"e\"\n");
    }

    #[test]
    fn dates_parse_shift_and_compare() {
        assert_eq!(days_from_civil(1970, 1, 1), 0);
        assert_eq!(days_from_civil(2026, 7, 25) - days_from_civil(2026, 1, 1), 205);
        // round trip through both directions
        for days in [0i64, 1, 12345, 20000, -1000] {
            let (y, m, d) = civil_from_days(days);
            assert_eq!(days_from_civil(y, m, d), days);
        }
        assert_eq!(strftime("%d/%m/%Y", 2026, 7, 25), "25/07/2026");
        assert_eq!(strftime("%Y-%m-%d", 2026, 7, 5), "2026-07-05");
    }

    #[test]
    fn impossible_dates_are_rejected() {
        let d = |s: &str| parse_date(&Value::Str(s.into()), 1, "test");
        assert!(d("2026-07-25").is_ok());
        assert!(d("२०२६-०७-२५").is_ok());        // Devanagari digits
        assert!(d("25/07/2026").is_ok());        // D/M/Y
        assert!(d("2026-02-29").is_err());       // not a leap year
        assert!(d("2024-02-29").is_ok());        // leap year
        assert!(d("2026-04-31").is_err());
        assert!(d("2026-13-01").is_err());
        assert!(d("कदापि").is_err());
    }
}
