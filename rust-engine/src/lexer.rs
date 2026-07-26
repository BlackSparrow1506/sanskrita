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
        "pratyekam" => "प्रत्येकम्",
        "iti" => "इति",
        "vargah" | "varga" => "वर्गः",
        "srja" | "srija" => "सृज",
        "ayam" => "अयम्",
        "prayata" | "prayat" => "प्रयत",
        "doshe" => "दोषे",
        "aanaya" | "anaya" => "आनय",
        "kshipa" | "kship" => "क्षिप",
        "vada" | "vad" => "वद",
        // kāraka role labels
        "kartaa" | "karta" => "कर्ता",
        "karma" => "कर्म",
        "karana" => "करण",
        "sampradaana" | "sampradana" => "सम्प्रदान",
        "apaadaana" | "apadana" => "अपादान",
        "adhikarana" => "अधिकरण",
        // builtins
        "yojaya" => "योजय",
        "apanaya" => "अपनय",
        "kunjikaah" | "kunjikah" | "kunjika" => "कुञ्जिकाः",
        "kramaya" => "क्रमय",
        "paridhih" | "paridhi" => "परिधिः",
        "prakarah" | "prakara" => "प्रकारः",
        "dairghyam" | "dairghya" => "दैर्घ्यम्",
        "vaakyam" | "vakyam" => "वाक्यम्",
        "sankhyaa" | "sankhya" => "सङ्ख्या",
        "aarambha" | "arambha" => "आरम्भ",
        other => other,
    }
}

const KEYWORDS: &[&str] = &[
    "मानय", "ध्रुव", "यदि", "अथ", "अन्यथा", "यावत्", "सत्यम्", "असत्यम्",
    "शून्यम्", "च", "वा", "न", "विरम", "अनुवर्त", "विधि", "फलम्",
    "प्रत्येकम्", "इति", "वर्गः", "सृज", "अयम्", "प्रयत", "दोषे", "आनय", "क्षिप",
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

// Devanagari block: U+0900..U+097F — but NOT the dandas (U+0964 ।, U+0965 ॥),
// which are punctuation (statement terminators), nor the digits U+0966..U+096F,
// which the number scanner owns. Getting this wrong glues '।' onto identifiers.
fn is_dev_danda(c: char) -> bool {
    c == '\u{0964}' || c == '\u{0965}'
}

fn is_devanagari(c: char) -> bool {
    ('\u{0900}'..='\u{097F}').contains(&c) && !is_dev_danda(c)
}

fn is_ident_start(c: char) -> bool {
    if is_dev_danda(c) || dev_digit(c).is_some() {
        return false;
    }
    c == '_' || c.is_alphabetic() || is_devanagari(c)
}

// identifier continuation also allows combining marks (matras, virama, nukta,
// anusvara/visarga) and digits — but still never a danda.
fn is_ident_cont(c: char) -> bool {
    if is_dev_danda(c) {
        return false;
    }
    is_ident_start(c) || c.is_ascii_digit() || dev_digit(c).is_some()
        || ('\u{0900}'..='\u{097F}').contains(&c)
}

pub fn lex(src: &str) -> Result<Vec<Token>, String> {
    // §10 #8: normalization is mandatory — visually identical text must be
    // identical to the engine (matches the Python reference's NFC call).
    let src = crate::nfc::normalize(src);
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
                // Collect the digits as typed (converting Devanagari to ASCII)
                // — no numeric conversion here, so arbitrary size and exact
                // decimals both stay possible downstream.
                let mut s = String::new();
                let mut seen_dot = false;
                while i < n {
                    let ch = chars[i];
                    if is_digit(ch) {
                        s.push(char::from(b'0' + digit_val(ch) as u8));
                        i += 1;
                    } else if ch == '.' && !seen_dot
                        && i + 1 < n && is_digit(chars[i + 1]) {
                        seen_dot = true;
                        s.push('.');
                        i += 1;
                    } else {
                        break;
                    }
                }
                out.push(Token::new(Tok::Num(s), line));
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
                if "+-*/%<>=(){}[],:.?".contains(c) {
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
        assert_eq!(toks[2].tok, Tok::Num("42".into()));
        assert_eq!(toks[4].tok, Tok::End);
    }

    #[test]
    fn decimal_literals() {
        let toks = lex("वद(०.१ + ३.१४१५९)।").unwrap();
        assert_eq!(toks[2].tok, Tok::Num("0.1".into()));
        assert_eq!(toks[4].tok, Tok::Num("3.14159".into()));
    }

    // a danda right after a number must not be read as a decimal point
    #[test]
    fn number_then_danda() {
        let toks = lex("मानय क = ५।").unwrap();
        assert_eq!(toks[3].tok, Tok::Num("5".into()));
        assert_eq!(toks[4].tok, Tok::End);
    }

    // huge literals are fine now — bignum handles them
    #[test]
    fn huge_literal_is_kept_exactly() {
        let toks = lex("मानय क = ९९९९९९९९९९९९९९९९९९९९९९९९।").unwrap();
        assert_eq!(toks[3].tok, Tok::Num("999999999999999999999999".into()));
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
        assert_eq!(toks[3].tok, Tok::Num("5".into()));
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

    // regression: the danda must NEVER be absorbed into an identifier or
    // keyword, even though U+0964 sits inside the Devanagari Unicode block.
    #[test]
    fn danda_not_part_of_identifier() {
        let toks = lex("क = इ।").unwrap();
        assert_eq!(toks[0].tok, Tok::Id("क".into()));
        assert_eq!(toks[2].tok, Tok::Id("इ".into()));   // not "इ।"
        assert_eq!(toks[3].tok, Tok::End);
    }

    #[test]
    fn danda_after_keyword() {
        // मानय(0) प(1) =(2) सत्यम्(3) ।(4) EOF(5)
        let toks = lex("मानय प = सत्यम्।").unwrap();
        assert_eq!(toks[3].tok, Tok::Kw("सत्यम्".into()));  // not Id("सत्यम्।")
        assert_eq!(toks[4].tok, Tok::End);
    }

    // identifiers may contain Devanagari digits and matras
    #[test]
    fn identifier_with_matras_and_digits() {
        let toks = lex("मानय वर्ष२ = ५।").unwrap();
        assert_eq!(toks[1].tok, Tok::Id("वर्ष२".into()));
    }

    // NFC: the same name typed two different ways must lex identically
    #[test]
    fn nfc_identifier_equivalence() {
        let a = lex("मानय \u{0958}मल = ५।").unwrap();          // क़ precomposed
        let b = lex("मानय \u{0915}\u{093C}मल = ५।").unwrap();  // क + nukta
        assert_eq!(a[1].tok, b[1].tok);
    }

}
