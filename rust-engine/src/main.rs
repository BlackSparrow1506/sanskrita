// main.rs — संस्कृता वेगः engine (native Rust).
// Slices 1-3: lexer + parser + evaluator. Runs core programs natively.
//
// Build:  cargo build --release
// Test:   cargo test
// Run:    cargo run -- program.सं
//         cargo run --release -- program.सं     (fast)

mod token;
mod err;
mod nfc;
mod lexer;
mod ast;
mod parser;
mod interp;

use std::env;
use std::fs;
use std::process;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.iter().any(|a| a == "--version") {
        println!("वेगः — संस्कृता native engine v{} (slices 1–3)", VERSION);
        return;
    }
    if args.len() < 2 || args.iter().any(|a| a == "--help") {
        eprintln!("प्रयोगः: sanskrita-veg <program.सं>");
        eprintln!("  --version   संस्करणम् / print version");
        eprintln!("  --help      एषा सूचना / this message");
        process::exit(if args.len() < 2 { 1 } else { 0 });
    }
    let path = &args[1];
    let src = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("सञ्चिका न प्राप्ता / cannot read {}: {}", path, e);
            process::exit(1);
        }
    };
    if let Err(e) = run(&src) {
        eprintln!("{}", e);
        process::exit(1);
    }
}

fn run(src: &str) -> Result<(), String> {
    let toks = lexer::lex(src)?;
    let stmts = parser::Parser::new(toks).program()?;
    interp::Interp::new().run(&stmts)
}
