// main.rs — संस्कृता वेगः engine (native Rust).
// Slices 1–7: the complete language — exact numbers, functions, collections,
// classes, try/catch, modules and the संस्कृतम् stdlib.
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
mod sanskritam;
mod stdlib;
mod parser;
mod precheck;
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
        println!("वेगः — संस्कृता native engine v{} (फलम्, Phase 3)", VERSION);
        return;
    }
    if args.iter().any(|a| a == "--help") {
        eprintln!("प्रयोगः: sanskrita-veg <program.सं> [आदेशचराः…]");
        eprintln!("  (सञ्चिकां विना)  अन्तःक्रियात्मकः — interactive REPL");
        eprintln!("  --version       संस्करणम् / print version");
        eprintln!("  --help          एषा सूचना / this message");
        process::exit(0);
    }
    if args.len() < 2 {
        // no program given → REPL, on the same big stack the runner uses
        let handle = thread::Builder::new()
            .stack_size(STACK_BYTES)
            .spawn(repl)
            .expect("could not start the interpreter thread");
        let _ = handle.join();
        return;
    }
    let path = args[1].clone();
    // everything after the program path belongs to the program (आदेशचराः)
    let prog_args: Vec<String> = args.iter().skip(2)
        .filter(|a| !a.starts_with("--")).cloned().collect();
    let dir = std::path::Path::new(&path).parent().map(|p| p.to_path_buf());
    let src = match fs::read_to_string(&path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("सञ्चिका न प्राप्ता / cannot read {}: {}", path, e);
            process::exit(1);
        }
    };

    let handle = thread::Builder::new()
        .stack_size(STACK_BYTES)
        .spawn(move || run(&src, dir, prog_args))
        .expect("could not start the interpreter thread");

    match handle.join() {
        Ok(Ok(())) => {}
        Ok(Err(e)) => {
            eprintln!("{}", err::render(&e));
            process::exit(1);
        }
        Err(_) => {
            eprintln!("आन्तरिकदोषः / interpreter thread panicked");
            process::exit(2);
        }
    }
}

/// अन्तःक्रियात्मकः — the वेगः REPL. State persists across lines, and a bare
/// expression shows its value (the reference engine's REPL behaves the same).
fn repl() {
    use std::io::{self, BufRead, Write};
    println!("ॐ  वेगः — संस्कृता native engine v{}", VERSION);
    println!("लिखतु आदेशम्; निर्गमाय Ctrl-D    (type code; Ctrl-D to exit)");

    let mut it = interp::Interp::new().with_limits(RUN_MAX_DEPTH, RUN_STACK_BUDGET);
    it.echo = true;
    it.source_dir = std::env::current_dir().ok();

    let stdin = io::stdin();
    let mut buf = String::new();
    let mut prompt = "॥ ";
    loop {
        print!("{}", prompt);
        let _ = io::stdout().flush();
        let mut line = String::new();
        match stdin.lock().read_line(&mut line) {
            Ok(0) => { println!("\nपुनर्मिलामः ।"); return; }
            Ok(_) => {}
            Err(_) => return,
        }
        buf.push_str(&line);
        // an unclosed block keeps reading — same rule as the reference REPL
        if buf.matches('{').count() > buf.matches('}').count() {
            prompt = "… ";
            continue;
        }
        prompt = "॥ ";
        let mut code = buf.trim().to_string();
        buf.clear();
        if code.is_empty() {
            continue;
        }
        if !(code.ends_with('।') || code.ends_with('॥')
             || code.ends_with('|') || code.ends_with('}')) {
            code.push('।');
        }
        let result = lexer::lex(&code)
            .and_then(|toks| parser::Parser::new(toks).program())
            .and_then(|stmts| {
                let problems = precheck::precheck(&stmts);
                if problems.is_empty() { it.run(&stmts) }
                else { Err(precheck::report(&problems)) }
            });
        if let Err(e) = result {
            println!("{}", err::render(&e));
        }
    }
}

fn run(src: &str, dir: Option<std::path::PathBuf>, prog_args: Vec<String>)
    -> Result<(), String>
{
    let toks = lexer::lex(src)?;
    let stmts = parser::Parser::new(toks).program()?;
    // §2b — annotations and obvious type mixing are checked BEFORE anything runs
    let problems = precheck::precheck(&stmts);
    if !problems.is_empty() {
        return Err(precheck::report(&problems));
    }
    let mut it = interp::Interp::new().with_limits(RUN_MAX_DEPTH, RUN_STACK_BUDGET);
    it.source_dir = dir;
    it.program_args = prog_args;
    it.run(&stmts)
}
