// token.rs — संस्कृता token types (वेगः engine)

#[derive(Debug, Clone, PartialEq)]
pub enum Tok {
    /// Numeric literal, kept as the DIGITS the user typed (ASCII, with an
    /// optional '.'), so the value layer can build an exact big integer or
    /// decimal. Never parsed into a float.
    Num(String),
    Str(String),     // "…"
    Kw(String),      // keyword (canonical Devanagari)
    Id(String),      // identifier
    Op(String),      // operator or punctuation
    End,             // danda । (or | or ॥)
    Eof,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub tok: Tok,
    pub line: usize,
}

impl Token {
    pub fn new(tok: Tok, line: usize) -> Self {
        Token { tok, line }
    }
}
