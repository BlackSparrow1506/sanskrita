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
const VAKYAKARMA: &[&str] = &["विभज", "संयोजय", "खोज", "प्रतिस्थापय", "अंश"];
const YADRCCHIKAM: &[&str] = &["अन्तरे", "वरय", "भिन्नम्"];
const KALAH: &[&str] = &["अद्य", "संप्रति", "वर्षः"];

fn members(module: &str) -> &'static [&'static str] {
    match module {
        "संस्कृतम्" => SANSKRITAM,
        "गणितम्" => GANITAM,
        "वाक्यकर्म" => VAKYAKARMA,
        "यादृच्छिकम्" => YADRCCHIKAM,
        "कालः" => KALAH,
        _ => &[],
    }
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

        // ---------------- कालः ----------------
        ("कालः", "अद्य") => Ok(Value::Str(dev_digits(&today()))),
        ("कालः", "संप्रति") => Ok(Value::Str(dev_digits(&clock()))),
        ("कालः", "वर्षः") => Ok(Value::int(year())),

        _ => Err(err(line,
            &format!("'{}' कोष्ठके '{}' नास्ति", module, name),
            &format!("module '{}' has no '{}'", module, name))),
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
