// err.rs — bilingual error scaffolding.
//
// STATUS (honest): the engine currently returns `String` errors that already
// have the correct two-line bilingual SHAPE, but print the line number in ASCII
// on the Sanskrit line where the Python reference prints Devanagari (५ vs 5).
// This module holds the correct formatting and becomes the engine's error type
// in slice 5, when errors also gain source spans. Tracked in AUDIT.md.
#![allow(dead_code)]

// bilingual errors, byte-identical in shape to the Python reference:
//
//   दोषः पङ्क्तौ ५ — <sanskrit>
//   Error at line 5 — <english>
//
// (Line numbers appear in Devanagari on the Sanskrit line, ASCII on the English
// line — exactly as sanskrita.py does.)

pub fn dev_digits(n: usize) -> String {
    let d = ['०', '१', '२', '३', '४', '५', '६', '७', '८', '९'];
    if n == 0 {
        return "०".to_string();
    }
    let mut s = String::new();
    let mut x = n;
    while x > 0 {
        s.insert(0, d[x % 10]);
        x /= 10;
    }
    s
}

#[derive(Debug, Clone)]
pub struct SError {
    pub line: usize,
    pub sa: String,
    pub en: String,
}

impl SError {
    pub fn new(line: usize, sa: impl Into<String>, en: impl Into<String>) -> Self {
        SError { line, sa: sa.into(), en: en.into() }
    }
}

impl std::fmt::Display for SError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "दोषः पङ्क्तौ {} — {}\nError at line {} — {}",
            dev_digits(self.line), self.sa, self.line, self.en
        )
    }
}

pub type SResult<T> = Result<T, SError>;
