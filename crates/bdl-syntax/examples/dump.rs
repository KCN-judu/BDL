//! `cargo run -p bdl-syntax --example dump -- [--formula] <file>`: print the
//! CST and diagnostics for a source file.

use std::io::Read;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let formula = args.iter().any(|a| a == "--formula");
    let path = args.iter().find(|a| !a.starts_with("--"));
    let mut src = String::new();
    match path {
        Some(p) => src = std::fs::read_to_string(p).expect("readable file"),
        None => {
            std::io::stdin().read_to_string(&mut src).expect("stdin");
        }
    }
    if formula {
        let p = bdl_syntax::parse_formula(&src);
        print!("{}", bdl_syntax::syntax::debug_tree(&p.syntax_node()));
        for e in p.errors() {
            println!("error {e}\n  {}", e.technical());
        }
        assert_eq!(p.text(), src);
    } else {
        let p = bdl_syntax::parse_module(&src);
        print!("{}", bdl_syntax::syntax::debug_tree(&p.syntax_node()));
        for e in p.errors() {
            println!("error {e}\n  {}", e.technical());
            if let Some(h) = &e.hint {
                println!("  hint: {h}");
            }
        }
        assert_eq!(p.text(), src);
    }
}
