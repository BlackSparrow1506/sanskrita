// nfc.rs — Unicode normalization for the Devanagari block, dependency-free.
//
// The Python reference calls unicodedata.normalize("NFC", src). For Devanagari
// the only characters this affects are the eight precomposed nukta letters
// U+0958..U+095F, which are *composition exclusions*: NFC DECOMPOSES them into
// base + nukta (U+093C). Without this, क़ typed as one codepoint and क + ़ typed
// as two would be different identifiers — the exact "ghost bug" §10 #8 forbids.
//
// Also decomposes the two candrabindu/vowel forms with nukta in 09DC..09DF? —
// those are Bengali, not our block, so they are left alone.

pub fn normalize(src: &str) -> String {
    let needs = src.chars().any(|c| ('\u{0958}'..='\u{095F}').contains(&c));
    if !needs {
        return src.to_string();
    }
    let mut out = String::with_capacity(src.len() + 8);
    for c in src.chars() {
        match c {
            '\u{0958}' => { out.push('\u{0915}'); out.push('\u{093C}'); } // क़
            '\u{0959}' => { out.push('\u{0916}'); out.push('\u{093C}'); } // ख़
            '\u{095A}' => { out.push('\u{0917}'); out.push('\u{093C}'); } // ग़
            '\u{095B}' => { out.push('\u{091C}'); out.push('\u{093C}'); } // ज़
            '\u{095C}' => { out.push('\u{0921}'); out.push('\u{093C}'); } // ड़
            '\u{095D}' => { out.push('\u{0922}'); out.push('\u{093C}'); } // ढ़
            '\u{095E}' => { out.push('\u{092B}'); out.push('\u{093C}'); } // फ़
            '\u{095F}' => { out.push('\u{092F}'); out.push('\u{093C}'); } // य़
            other => out.push(other),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn precomposed_nukta_decomposes() {
        let pre = "\u{0958}मल";                       // क़मल as one codepoint
        let dec = "\u{0915}\u{093C}मल";               // क + ़ + मल
        assert_eq!(normalize(pre), dec);
        assert_eq!(normalize(dec), dec);              // idempotent
    }

    #[test]
    fn untouched_when_not_needed() {
        assert_eq!(normalize("नमस्ते जगत्"), "नमस्ते जगत्");
    }
}
