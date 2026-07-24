// lexer.rs — संस्कृता lexer (वेगः engine)
// Mirrors sanskrita.py's lex(): NFC is assumed on input; converts Devanagari
// digits, handles danda, strings, comments, operators, keywords, identifiers.

use crate::token::{Tok, Token};

// roman alias -> canonical Devanagari keyword/builtin (subset of ALIASES)
fn alias(word: &str) -> &str {
    match word {
        "maanaya" | "manaya" | "manay" => "मानय",
        "dhruva" | "dhruv" => "ध्रुव",
        "yadi" => "यदि",
        "atha" => "अथ",
        "anyathaa" | "anyatha" => "अन्यथा",
        "yaavat" | "yavat" => "यावत्",
        "satyam" => "सत्यम्",
        "asatyam" => "असत्यम्",
        "shuunyam" | "shunyam" => "शून्यम्",
        "cha" | "ca" => "च",
        "vaa" => "वा",
        "na" => "न",
        "virama" | "viram" => "विरम",
        "anuvarta" | "anuvart" => "अनुवर्त",
        "vidhi" => "विधि",
        "phalam" => "फलम्",
        "vada" | "vad" => "वद",
        other => other,
    }
}

const KEYWORDS: &[&str] = &[
    "मानय", "ध्रुव", "यदि", "अथ", "अन्यथा", "यावत्", "सत्यम्", "असत्यम्",
    "शून्यम्", "च", "वा", "न", "विरम", "अनुवर्त", "विधि", "फलम्",
];

fn dev_digit(c: char) -> Option<i64> {
    match c {
        '०' => Some(0), '१' => Some(1), '२' => Some(2), '३' => Some(3),
        '४' => Some(4), '५' => Some(5), '६' => Some(6), '७' => Some(7),
        '८' => Some(8), '९' => Some(9),
        _ => None,
    }
}

fn is_digit(c: char) -> bool {
    c.is_ascii_digit() || dev_digit(c).is_some()
}

fn digit_val(c: char) -> i64 {
    if let Some(d) = dev_digit(c) { d } else { (c as i64) - ('0' as i64) }
}

// Devanagari block: U+0900..U+097F
fn is_devanagari(c: char) -> bool {
    ('\u{0900}'..='\u{097F}').contains(&c)
}

fn is_ident_start(c: char) -> bool {
    c == '_' || c.is_alphabetic() || is_devanagari(c)
}

fn is_ident_cont(c: char) -> bool {
    is_ident_start(c) || c.is_ascii_digit()
        // combining marks (matras) are part of a Devanagari cluster
        || ('\u{0900}'..='\u{097F}').contains(&c)
}

pub fn lex(src: &str) -> Result<Vec<Token>, String> {
    let chars: Vec<char> = src.chars().collect();
    let n = chars.len();
    let mut i = 0usize;
    let mut line = 1usize;
    let mut out: Vec<Token> = Vec::new();

    while i < n {
        let c = chars[i];
        match c {
            '\n' => { line += 1; i += 1; }
            ' ' | '\t' | '\r' => { i += 1; }
            '#' => { while i < n && chars[i] != '\n' { i += 1; } }
            '।' | '॥' | '|' => { out.push(Token::new(Tok::End, line)); i += 1; }
            '"' => {
                i += 1;
                let start_line = line;
                let mut buf = String::new();
                while i < n && chars[i] != '"' {
                    if chars[i] == '\\' && i + 1 < n {
                        let e = chars[i + 1];
                        buf.push(match e {
                            'n' => '\n', 't' => '\t', '"' => '"', '\\' => '\\',
                            other => other,
                        });
                        i += 2;
                    } else {
                        if chars[i] == '\n' { line += 1; }
                        buf.push(chars[i]);
                        i += 1;
                    }
                }
                if i >= n {
                    return Err(format!(
                        "दोषः पङ्क्तौ {} — अपूर्णं वाक्यम् / unterminated string", start_line));
                }
                i += 1; // closing quote
                out.push(Token::new(Tok::Str(buf), line));
            }
            _ if is_digit(c) => {
                let mut val: i64 = 0;
                while i < n && is_digit(chars[i]) {
                    val = val * 10 + digit_val(chars[i]);
                    i += 1;
                }
                // decimals belong to a later slice; stop at '.' here
                out.push(Token::new(Tok::Num(val), line));
            }
            _ => {
                // two-char operators
                if i + 1 < n {
                    let two: String = [chars[i], chars[i + 1]].iter().collect();
                    if two == "==" || two == "!=" || two == "<=" || two == ">=" {
                        out.push(Token::new(Tok::Op(two), line));
                        i += 2;
                        continue;
                    }
                }
                if "+-*/%<>=(){}[],:.".contains(c) {
                    out.push(Token::new(Tok::Op(c.to_string()), line));
                    i += 1;
                    continue;
                }
                if is_ident_start(c) {
                    let start = i;
                    i += 1;
                    while i < n && is_ident_cont(chars[i]) {
                        i += 1;
                    }
                    let raw: String = chars[start..i].iter().collect();
                    // mixed-script guard
                    let has_dev = raw.chars().any(is_devanagari);
                    let has_lat = raw.chars().any(|c| c.is_ascii_alphabetic());
                    if has_dev && has_lat {
                        return Err(format!(
                            "दोषः पङ्क्तौ {} — मिश्रलिपि-नाम '{}' / mixed-script name", line, raw));
                    }
                    let word = alias(&raw).to_string();
                    if KEYWORDS.contains(&word.as_str()) {
                        out.push(Token::new(Tok::Kw(word), line));
                    } else {
                        out.push(Token::new(Tok::Id(word), line));
                    }
                    continue;
                }
                return Err(format!(
                    "दोषः पङ्क्तौ {} — अज्ञातं चिह्नम् '{}' / unknown character", line, c));
            }
        }
    }
    out.push(Token::new(Tok::Eof, line));
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn digits_devanagari_and_ascii() {
        let toks = lex("वद(४२)।").unwrap();
        // वद ( ४२ ) ।  EOF
        assert!(matches!(toks[0].tok, Tok::Id(_)));
        assert_eq!(toks[2].tok, Tok::Num(42));
        assert_eq!(toks[4].tok, Tok::End);
    }

    #[test]
    fn keyword_vs_identifier() {
        let toks = lex("मानय क = ५।").unwrap();
        assert_eq!(toks[0].tok, Tok::Kw("मानय".into()));
        assert_eq!(toks[1].tok, Tok::Id("क".into()));
    }

    #[test]
    fn roman_aliases() {
        let toks = lex("manay k = 5|").unwrap();
        assert_eq!(toks[0].tok, Tok::Kw("मानय".into()));
        assert_eq!(toks[3].tok, Tok::Num(5));
        assert_eq!(toks[4].tok, Tok::End);
    }

    #[test]
    fn mixed_script_rejected() {
        assert!(lex("मानय नामx = ५।").is_err());
    }

    #[test]
    fn string_and_comment() {
        let toks = lex("# hi\nवद(\"नमस्ते\")।").unwrap();
        assert_eq!(toks[0].tok, Tok::Id("वद".into()));
        assert_eq!(toks[2].tok, Tok::Str("नमस्ते".into()));
    }
}
