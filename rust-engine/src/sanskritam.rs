// sanskritam.rs — the संस्कृतम् linguistics library, natively in Rust.
//
// A port of the Python reference's module: akṣara splitting, laghu/guru
// weights, meter detection, Devanagari↔IAST transliteration, and sandhi.
// This is the capability no other programming language has built in, so it
// must work identically in both engines.

pub const VIRAMA: char = '\u{094D}';

const CONS: &[(char, &str)] = &[
    ('क', "k"), ('ख', "kh"), ('ग', "g"), ('घ', "gh"), ('ङ', "ṅ"),
    ('च', "c"), ('छ', "ch"), ('ज', "j"), ('झ', "jh"), ('ञ', "ñ"),
    ('ट', "ṭ"), ('ठ', "ṭh"), ('ड', "ḍ"), ('ढ', "ḍh"), ('ण', "ṇ"),
    ('त', "t"), ('थ', "th"), ('द', "d"), ('ध', "dh"), ('न', "n"),
    ('प', "p"), ('फ', "ph"), ('ब', "b"), ('भ', "bh"), ('म', "m"),
    ('य', "y"), ('र', "r"), ('ल', "l"), ('व', "v"),
    ('श', "ś"), ('ष', "ṣ"), ('स', "s"), ('ह', "h"), ('ळ', "ḷ"),
];

const VOWELS: &[(char, &str)] = &[
    ('अ', "a"), ('आ', "ā"), ('इ', "i"), ('ई', "ī"), ('उ', "u"), ('ऊ', "ū"),
    ('ऋ', "ṛ"), ('ॠ', "ṝ"), ('ए', "e"), ('ऐ', "ai"), ('ओ', "o"), ('औ', "au"),
];

const MATRAS: &[(char, &str)] = &[
    ('ा', "ā"), ('ि', "i"), ('ी', "ī"), ('ु', "u"), ('ू', "ū"),
    ('ृ', "ṛ"), ('ॄ', "ṝ"), ('े', "e"), ('ै', "ai"), ('ो', "o"), ('ौ', "au"),
];

const SIGNS: &[(char, &str)] = &[
    ('ं', "ṃ"), ('ः', "ḥ"), ('ँ', "m̐"), ('ऽ', "'"),
];

/// vowel → its mātrā sign ('' for the inherent अ)
const MATRA_OF: &[(char, &str)] = &[
    ('अ', ""), ('आ', "ा"), ('इ', "ि"), ('ई', "ी"), ('उ', "ु"), ('ऊ', "ू"),
    ('ऋ', "ृ"), ('ॠ', "ॄ"), ('ए', "े"), ('ऐ', "ै"), ('ओ', "ो"), ('औ', "ौ"),
];

const LONG: &str = "आईऊॠएऐओऔाीूॄेैोौ";
const GHOSA: &str = "गघङजझञडढणदधनबभमयरलवह"; // voiced consonants

fn is_cons(c: char) -> bool {
    CONS.iter().any(|(k, _)| *k == c)
}

fn is_vowel(c: char) -> bool {
    VOWELS.iter().any(|(k, _)| *k == c)
}

fn is_matra(c: char) -> bool {
    MATRAS.iter().any(|(k, _)| *k == c)
}

fn look(table: &[(char, &str)], c: char) -> Option<&'static str> {
    table.iter().find(|(k, _)| *k == c).map(|(_, v)| *v)
}

fn matra_of(v: char) -> &'static str {
    look(MATRA_OF, v).unwrap_or("")
}

fn matra_to_vowel(m: char) -> Option<char> {
    MATRA_OF.iter().find(|(_, s)| s.chars().next() == Some(m)).map(|(v, _)| *v)
}

/// Split Devanagari text into akṣaras (syllables).
pub fn aksharani(text: &str) -> Vec<String> {
    let chars: Vec<char> = text.chars().collect();
    let n = chars.len();
    let mut out: Vec<String> = Vec::new();
    let mut i = 0;
    while i < n {
        let c = chars[i];
        if is_cons(c) {
            let mut cluster = String::new();
            cluster.push(c);
            i += 1;
            while i + 1 < n && chars[i] == VIRAMA && is_cons(chars[i + 1]) {
                cluster.push(chars[i]);
                cluster.push(chars[i + 1]);
                i += 2;
            }
            if i < n && chars[i] == VIRAMA {
                cluster.push(chars[i]);
                i += 1;
            } else if i < n && is_matra(chars[i]) {
                cluster.push(chars[i]);
                i += 1;
            }
            while i < n && "ंःँ".contains(chars[i]) {
                cluster.push(chars[i]);
                i += 1;
            }
            out.push(cluster);
        } else if is_vowel(c) {
            let mut cluster = String::new();
            cluster.push(c);
            i += 1;
            while i < n && "ंःँ".contains(chars[i]) {
                cluster.push(chars[i]);
                i += 1;
            }
            out.push(cluster);
        } else {
            i += 1; // spaces, danda, anything else
        }
    }
    // a trailing halanta cluster belongs to the previous akṣara (एवम् → …वम्)
    let mut merged: Vec<String> = Vec::new();
    for a in out {
        if a.ends_with(VIRAMA) && !merged.is_empty() {
            let last = merged.len() - 1;
            merged[last].push_str(&a);
        } else {
            merged.push(a);
        }
    }
    merged
}

/// Laghu (ल) / guru (ग) weight of each akṣara.
pub fn matrah(text: &str) -> Vec<String> {
    let aks = aksharani(text);
    let mut out = Vec::with_capacity(aks.len());
    for (idx, a) in aks.iter().enumerate() {
        let mut heavy = a.chars().any(|c| LONG.contains(c))
            || a.contains('ं') || a.contains('ः');
        if !heavy && a.ends_with(VIRAMA) {
            heavy = true;
        }
        if !heavy {
            if let Some(next) = aks.get(idx + 1) {
                if next.contains(VIRAMA) {
                    heavy = true;
                }
            }
        }
        out.push(if heavy { "ग".to_string() } else { "ल".to_string() });
    }
    out
}

/// Identify a verse's meter by its akṣara count.
pub fn chandah(text: &str) -> String {
    let count = aksharani(text).len();
    let name = match count {
        24 => "गायत्री",
        32 => "अनुष्टुभ्",
        44 => "त्रिष्टुभ्",
        48 => "जगती",
        _ => "अज्ञातम्",
    };
    format!("{} (अक्षराणि: {})", name, crate::interp::dev_digits(&count.to_string()))
}

/// Devanagari → IAST.
pub fn romanaya(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let n = chars.len();
    let mut out = String::new();
    let mut i = 0;
    while i < n {
        let c = chars[i];
        if let Some(r) = look(CONS, c) {
            out.push_str(r);
            if i + 1 < n && chars[i + 1] == VIRAMA {
                i += 2;
                continue;
            }
            if i + 1 < n {
                if let Some(m) = look(MATRAS, chars[i + 1]) {
                    out.push_str(m);
                    i += 2;
                    continue;
                }
            }
            out.push('a');
            i += 1;
            continue;
        }
        if let Some(r) = look(VOWELS, c) {
            out.push_str(r);
            i += 1;
            continue;
        }
        if let Some(r) = look(SIGNS, c) {
            out.push_str(r);
            i += 1;
            continue;
        }
        if ('०'..='९').contains(&c) {
            out.push(char::from(b'0' + (c as u32 - '०' as u32) as u8));
            i += 1;
            continue;
        }
        match c {
            '॥' => out.push_str("||"),
            '।' => out.push('|'),
            other => out.push(other),
        }
        i += 1;
    }
    out
}

/// IAST → Devanagari.
pub fn devanagaraya(text: &str) -> String {
    // longest-match first
    let mut cons: Vec<(&str, char)> = CONS.iter().map(|(d, r)| (*r, *d)).collect();
    cons.sort_by_key(|(r, _)| std::cmp::Reverse(r.len()));
    let mut vows: Vec<(&str, char)> = VOWELS.iter().map(|(d, r)| (*r, *d)).collect();
    vows.sort_by_key(|(r, _)| std::cmp::Reverse(r.len()));

    let mut out = String::new();
    let mut rest = text;
    while !rest.is_empty() {
        if let Some(stripped) = rest.strip_prefix('ṃ') {
            out.push('ं');
            rest = stripped;
            continue;
        }
        if let Some(stripped) = rest.strip_prefix('ḥ') {
            out.push('ः');
            rest = stripped;
            continue;
        }
        if let Some((r, d)) = cons.iter().find(|(r, _)| rest.starts_with(*r)) {
            out.push(*d);
            rest = &rest[r.len()..];
            if let Some((vr, vd)) = vows.iter().find(|(vr, _)| rest.starts_with(*vr)) {
                out.push_str(matra_of(*vd));
                rest = &rest[vr.len()..];
            } else {
                out.push(VIRAMA);
            }
            continue;
        }
        if let Some((r, d)) = vows.iter().find(|(r, _)| rest.starts_with(*r)) {
            out.push(*d);
            rest = &rest[r.len()..];
            continue;
        }
        let c = rest.chars().next().unwrap();
        if c == '|' {
            out.push('।');
        } else if c.is_ascii_digit() {
            out.push(char::from_u32('०' as u32 + (c as u32 - '0' as u32)).unwrap_or(c));
        } else {
            out.push(c);
        }
        rest = &rest[c.len_utf8()..];
    }
    out
}

fn vclass(v: char) -> Option<&'static str> {
    Some(match v {
        'अ' | 'आ' => "a",
        'इ' | 'ई' => "i",
        'उ' | 'ऊ' => "u",
        'ऋ' => "r",
        'ए' => "e",
        'ऐ' => "ai",
        'ओ' => "o",
        'औ' => "au",
        _ => return None,
    })
}

/// Join two words by sandhi: vowel rules plus the common visarga rules.
pub fn sandhaya(a: &str, b: &str) -> String {
    let a = a.trim();
    let b = b.trim();
    if a.is_empty() || b.is_empty() {
        return format!("{}{}", a, b);
    }
    let a_chars: Vec<char> = a.chars().collect();
    let first_b = b.chars().next().unwrap();
    let rest_b: String = b.chars().skip(1).collect();

    // ---- visarga sandhi ----
    if a.ends_with('ः') && a_chars.len() >= 2 {
        let after_aa = a_chars[a_chars.len() - 2] == 'ा';
        let stem: String = a_chars[..a_chars.len() - 1].iter().collect();
        if !after_aa {
            if first_b == 'अ' {
                return format!("{}ोऽ{}", stem, rest_b);
            }
            if GHOSA.contains(first_b) || is_vowel(first_b) {
                return format!("{}ो {}", stem, b);
            }
        } else if GHOSA.contains(first_b) || is_vowel(first_b) {
            return format!("{} {}", stem, b);
        }
        if "चछ".contains(first_b) {
            return format!("{}श{}{}", stem, VIRAMA, b);
        }
        if "टठ".contains(first_b) {
            return format!("{}ष{}{}", stem, VIRAMA, b);
        }
        if "तथ".contains(first_b) {
            return format!("{}स{}{}", stem, VIRAMA, b);
        }
        return format!("{} {}", a, b);
    }

    // ---- vowel sandhi ----
    if !is_vowel(first_b) {
        return format!("{}{}", a, b);
    }
    let last = a_chars[a_chars.len() - 1];
    let (v1, base, ends_c): (char, String, bool) = if let Some(v) = matra_to_vowel(last) {
        (v, a_chars[..a_chars.len() - 1].iter().collect(), true)
    } else if is_cons(last) {
        ('अ', a.to_string(), true)
    } else if is_vowel(last) {
        (last, a_chars[..a_chars.len() - 1].iter().collect(), false)
    } else {
        return format!("{}{}", a, b);
    };
    let c1 = vclass(v1);
    let c2 = vclass(first_b);
    let attach = |vowel: char| -> String {
        if ends_c {
            format!("{}{}", base, matra_of(vowel))
        } else {
            format!("{}{}", base, vowel)
        }
    };
    if c1.is_some() && c1 == c2 && matches!(c1, Some("a") | Some("i") | Some("u")) {
        let long = match c1 {
            Some("a") => 'आ',
            Some("i") => 'ई',
            _ => 'ऊ',
        };
        return format!("{}{}", attach(long), rest_b);
    }
    if c1 == Some("a") {
        match c2 {
            Some("i") => return format!("{}{}", attach('ए'), rest_b),
            Some("u") => return format!("{}{}", attach('ओ'), rest_b),
            Some("r") => return format!("{}र{}{}", attach('अ'), VIRAMA, rest_b),
            Some("e") | Some("ai") => return format!("{}{}", attach('ऐ'), rest_b),
            Some("o") | Some("au") => return format!("{}{}", attach('औ'), rest_b),
            _ => {}
        }
    }
    if c1 == Some("i") && c2 != Some("i") {
        let v = if ends_c { format!("{}{}", base, VIRAMA) } else { base.clone() };
        return format!("{}य{}{}", v, matra_of(first_b), rest_b);
    }
    if c1 == Some("u") && c2 != Some("u") {
        let v = if ends_c { format!("{}{}", base, VIRAMA) } else { base.clone() };
        return format!("{}व{}{}", v, matra_of(first_b), rest_b);
    }
    if matches!(c1, Some("e") | Some("o")) && c2 == Some("a") {
        return format!("{}ऽ{}", a, rest_b);
    }
    format!("{}{}", a, b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn syllables() {
        assert_eq!(aksharani("नमस्ते"), vec!["न", "म", "स्ते"]);
        assert_eq!(aksharani("संस्कृता").len(), 3);
        assert_eq!(aksharani("धर्मक्षेत्रे कुरुक्षेत्रे"),
                   vec!["ध", "र्म", "क्षे", "त्रे", "कु", "रु", "क्षे", "त्रे"]);
    }

    #[test]
    fn meter_of_the_gita() {
        let sloka = "धर्मक्षेत्रे कुरुक्षेत्रे समवेता युयुत्सवः \
                     मामकाः पाण्डवाश्चैव किमकुर्वत सञ्जय";
        assert!(chandah(sloka).starts_with("अनुष्टुभ्"), "got {}", chandah(sloka));
    }

    #[test]
    fn transliteration_both_ways() {
        assert_eq!(romanaya("संस्कृता श्रेष्ठा भाषा"), "saṃskṛtā śreṣṭhā bhāṣā");
        assert_eq!(devanagaraya("yoga dharma karma mokṣa"), "योग धर्म कर्म मोक्ष");
    }

    #[test]
    fn vowel_sandhi() {
        assert_eq!(sandhaya("देव", "आलयः"), "देवालयः");
        assert_eq!(sandhaya("महा", "ईशः"), "महेशः");
        assert_eq!(sandhaya("गुरु", "उपदेशः"), "गुरूपदेशः");
        assert_eq!(sandhaya("इति", "एवम्"), "इत्येवम्");
        assert_eq!(sandhaya("विद्या", "अर्थी"), "विद्यार्थी");
        assert_eq!(sandhaya("ते", "अपि"), "तेऽपि");
    }

    #[test]
    fn visarga_sandhi() {
        assert_eq!(sandhaya("रामः", "गच्छति"), "रामो गच्छति");
        assert_eq!(sandhaya("रामः", "अस्ति"), "रामोऽस्ति");
        assert_eq!(sandhaya("रामः", "च"), "रामश्च");
        assert_eq!(sandhaya("देवाः", "गच्छन्ति"), "देवा गच्छन्ति");
        assert_eq!(sandhaya("रामः", "करोति"), "रामः करोति");
    }

    #[test]
    fn laghu_guru() {
        assert_eq!(matrah("धर्मक्षेत्रे"), vec!["ग", "ग", "ग", "ग"]);
    }
}
