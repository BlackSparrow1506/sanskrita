// main.rs — संस्कृता वेगः engine (native Rust).
// Slices 1–6: lexer, parser, evaluator — exact numbers, functions,
// collections, classes, try/catch.
//
// Build:  cargo build --release
// Test:   cargo test
// Run:    cargo run --release -- program.सं

mod token;
mod bigint;
mod decimal;
mod err;
mod nfc;
mod lexer;
mod ast;
mod value;
mod parser;
mod interp;

use std::env;
use std::fs;
use std::process;
use std::thread;

const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Interpreters recurse natively, so the depth a user program can reach is
/// bounded by the OS stack. We run the program on a thread with a large,
/// explicit stack and set the guard well inside it — deep recursion then
/// works, and anything runaway still gets a clean bilingual error instead of
/// a crash. (CPython and rustc use the same technique.)
const STACK_BYTES: usize = 256 * 1024 * 1024;
const RUN_MAX_DEPTH: usize = 1_000_000;
/// Budget is kept well below STACK_BYTES so the guard always fires first.
const RUN_STACK_BUDGET: usize = 192 * 1024 * 1024;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.iter().any(|a| a == "--version") {
        println!("वेगः — संस्कृता native engine v{} (slices 1–6)", VERSION);
        return;
    }
    if args.len() < 2 || args.iter().any(|a| a == "--help") {
        eprintln!("प्रयोगः: sanskrita-veg <program.सं>");
        eprintln!("  --version   संस्करणम् / print version");
        eprintln!("  --help      एषा सूचना / this message");
        process::exit(if args.len() < 2 { 1 } else { 0 });
    }
    let path = args[1].clone();
    let src = match fs::read_to_string(&path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("सञ्चिका न प्राप्ता / cannot read {}: {}", path, e);
            process::exit(1);
        }
    };

    let handle = thread::Builder::new()
        .stack_size(STACK_BYTES)
        .spawn(move || run(&src))
        .expect("could not start the interpreter thread");

    match handle.join() {
        Ok(Ok(())) => {}
        Ok(Err(e)) => {
            eprintln!("{}", e);
            process::exit(1);
        }
        Err(_) => {
            eprintln!("आन्तरिकदोषः / interpreter thread panicked");
            process::exit(2);
        }
    }
}

fn run(src: &str) -> Result<(), String> {
    let toks = lexer::lex(src)?;
    let stmts = parser::Parser::new(toks).program()?;
    interp::Interp::new()
        .with_limits(RUN_MAX_DEPTH, RUN_STACK_BUDGET)
        .run(&stmts)
}
