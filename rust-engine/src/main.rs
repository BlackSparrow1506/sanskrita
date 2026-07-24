// main.rs — संस्कृता वेगः engine (native Rust). Slice 1: lexer demo.
// This will grow into a full interpreter passing the conformance suite.
//
// Build:  cargo build --release
// Test:   cargo test
// Run:    cargo run -- program.सं        (currently: prints the token stream)

mod token;
mod lexer;

use std::env;
use std::fs;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("प्रयोगः: sanskrita-veg <program.सं>");
        eprintln!("(वेगः engine — slice 1: lexer. Full interpreter in progress.)");
        process::exit(1);
    }
    let path = &args[1];
    let src = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("सञ्चिका न प्राप्ता / cannot read {}: {}", path, e);
            process::exit(1);
        }
    };
    // NFC normalization would go here (needs unicode-normalization crate; added
    // when we wire dependencies — for now inputs are assumed NFC).
    match lexer::lex(&src) {
        Ok(toks) => {
            for t in &toks {
                println!("{:>4}  {:?}", t.line, t.tok);
            }
        }
        Err(e) => {
            eprintln!("{}", e);
            process::exit(1);
        }
    }
}
