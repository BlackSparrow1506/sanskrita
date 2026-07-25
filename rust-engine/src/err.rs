// err.rs — bilingual errors with a kind and a call stack.
//
// The engine's error channel is `Result<_, String>`, which is cheap and
// pervasive: over a hundred sites return one. Rather than rewrite every one of
// them to carry a struct, an error is *encoded into* that string with a marker
// no source file will contain, and decoded at the three places that need the
// structure: प्रयत (to build the value the catch block binds), the call
// boundary (to push a stack frame), and the top level (to print).
//
// The human-readable form is produced by Display and is byte-identical in shape
// to the Python reference:
//
//   दोषः पङ्क्तौ ५ — <sanskrit>
//   Error at line 5 — <english>
//
//   अनुरेखा (नवीनतमम् आह्वानम् अन्ते) / traceback (most recent call last):
//       विधि 'बाह्यः' — पङ्क्तिः ४
//       → पङ्क्तिः ३: <sanskrit>
//
// Line numbers are Devanagari on the Sanskrit line and ASCII on the English
// line — exactly as sanskrita.py does.

/// The kinds a program can branch on. Short and stable on purpose: code that
/// says `यदि (त्रु.प्रकारः == "गणितदोषः")` must keep working across releases.
pub const KINDS: &[&str] = &["दोषः", "नामदोषः", "प्रकारदोषः", "गणितदोषः",
                             "सीमादोषः", "व्याकरणदोषः", "आयातदोषः", "स्वयंदोषः"];

const MARK: &str = "\u{1}\u{2}सं\u{2}\u{1}";
const SEP: char = '\u{1}';
const FRAME_SEP: char = '\u{2}';
const FRAME_JOIN: char = '\u{3}';

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
    pub kind: String,
    /// (function name, call-site line), innermost first — each विधि the error
    /// escapes appends its own frame on the way out.
    pub trace: Vec<(String, usize)>,
}

impl SError {
    pub fn new(line: usize, sa: impl Into<String>, en: impl Into<String>,
               kind: &str) -> Self {
        let kind = if KINDS.contains(&kind) { kind } else { "दोषः" };
        SError { line, sa: sa.into(), en: en.into(), kind: kind.to_string(),
                 trace: Vec::new() }
    }

    /// Pack into the engine's `String` error channel.
    pub fn encode(&self) -> String {
        let frames: Vec<String> = self.trace.iter()
            .map(|(n, l)| format!("{}{}{}", n, FRAME_SEP, l)).collect();
        format!("{m}{s}{k}{s}{l}{s}{sa}{s}{en}{s}{tr}",
                m = MARK, s = SEP, k = self.kind, l = self.line,
                sa = self.sa, en = self.en, tr = frames.join(&FRAME_JOIN.to_string()))
    }

    /// Unpack. Anything that was not produced by `encode` — a lexer or parser
    /// message, say — still becomes a usable SError; we simply know no more
    /// about it than its text.
    pub fn decode(s: &str) -> SError {
        if !s.starts_with(MARK) {
            let first = s.lines().next().unwrap_or(s);
            let sa = match first.split_once(" — ") {
                Some((_, rest)) => rest.to_string(),
                None => first.to_string(),
            };
            let en = match s.lines().nth(1) {
                Some(l) => match l.split_once(" — ") {
                    Some((_, rest)) => rest.to_string(),
                    None => l.to_string(),
                },
                None => sa.clone(),
            };
            return SError::new(0, sa, en, "दोषः");
        }
        let fields: Vec<&str> = s[MARK.len()..].split(SEP).collect();
        // fields[0] is the empty piece before the first separator
        let kind = fields.get(1).copied().unwrap_or("दोषः").to_string();
        let line = fields.get(2).and_then(|v| v.parse().ok()).unwrap_or(0);
        let sa = fields.get(3).copied().unwrap_or("").to_string();
        let en = fields.get(4).copied().unwrap_or("").to_string();
        let mut trace = Vec::new();
        if let Some(rest) = fields.get(5) {
            for f in rest.split(FRAME_JOIN).filter(|f| !f.is_empty()) {
                if let Some((n, l)) = f.split_once(FRAME_SEP) {
                    trace.push((n.to_string(), l.parse().unwrap_or(0)));
                }
            }
        }
        SError { line, sa, en, kind, trace }
    }

    /// Add the frame of a विधि the error is escaping.
    pub fn with_frame(mut self, name: &str, line: usize) -> Self {
        self.trace.push((name.to_string(), line));
        self
    }
}

impl std::fmt::Display for SError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "दोषः पङ्क्तौ {} — {}\nError at line {} — {}",
               dev_digits(self.line), self.sa, self.line, self.en)?;
        if !self.trace.is_empty() {
            write!(f, "\n\nअनुरेखा (नवीनतमम् आह्वानम् अन्ते) / \
                       traceback (most recent call last):")?;
            for (name, ln) in self.trace.iter().rev() {
                write!(f, "\n    विधि '{}' — पङ्क्तिः {}", name, dev_digits(*ln))?;
            }
            write!(f, "\n    → पङ्क्तिः {}: {}", dev_digits(self.line), self.sa)?;
        }
        Ok(())
    }
}

/// True when this string came out of `encode`.
pub fn is_encoded(raw: &str) -> bool {
    raw.starts_with(MARK)
}

/// Render whatever the engine produced, structured or not.
pub fn render(raw: &str) -> String {
    if is_encoded(raw) {
        SError::decode(raw).to_string()
    } else {
        raw.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_through_the_string_channel() {
        let e = SError::new(7, "शून्येन भागो न शक्यः", "division by zero",
                            "गणितदोषः")
            .with_frame("आन्तरः", 3)
            .with_frame("बाह्यः", 9);
        let back = SError::decode(&e.encode());
        assert_eq!(back.line, 7);
        assert_eq!(back.kind, "गणितदोषः");
        assert_eq!(back.sa, "शून्येन भागो न शक्यः");
        assert_eq!(back.en, "division by zero");
        assert_eq!(back.trace, vec![("आन्तरः".to_string(), 3),
                                    ("बाह्यः".to_string(), 9)]);
    }

    #[test]
    fn plain_messages_still_decode() {
        let raw = "दोषः पङ्क्तौ ५ — किमपि\nError at line 5 — something";
        let e = SError::decode(raw);
        assert_eq!(e.sa, "किमपि");
        assert_eq!(e.en, "something");
        assert_eq!(e.kind, "दोषः");
        assert!(e.trace.is_empty());
    }

    #[test]
    fn an_unknown_kind_falls_back() {
        assert_eq!(SError::new(1, "a", "b", "किमपि-अन्यत्").kind, "दोषः");
    }

    #[test]
    fn display_shows_the_traceback_outermost_first() {
        let e = SError::new(3, "स", "s", "गणितदोषः")
            .with_frame("आन्तरः", 2)
            .with_frame("बाह्यः", 4);
        let text = e.to_string();
        let outer = text.find("बाह्यः").unwrap();
        let inner = text.find("आन्तरः").unwrap();
        assert!(outer < inner, "outermost frame must print first:\n{}", text);
    }

    #[test]
    fn a_message_with_no_frames_prints_exactly_as_before() {
        let e = SError::new(5, "किमपि", "something", "दोषः");
        assert_eq!(e.to_string(),
                   "दोषः पङ्क्तौ ५ — किमपि\nError at line 5 — something");
    }
}
