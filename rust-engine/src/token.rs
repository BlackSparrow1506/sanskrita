// token.rs — संस्कृता token types (वेगः engine)

#[derive(Debug, Clone, PartialEq)]
pub enum Tok {
    Num(i64),        // whole-number literal (subset: integers first; decimals later)
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
