mod ast;
mod lexer;
mod parser;
mod generator;
mod semantic_analysis;

use std::env;
use std::fs;
use std::path::Path;

use lexer::tokenize;
use parser::Parser;
use generator::generate;
use semantic_analysis::analyze;

fn main() {
    let input_path = read_main();
    let source = fs::read_to_string(&input_path)
        .expect("Failed to read input file");

    let tokens = tokenize(&source);
    let mut parser = Parser::new(tokens);
    let mut ast = parser.parse_program();
    ast.simplify();
    ast.to_base();


    let ast = analyze(ast);
    let json = generate(&ast);
    write_to_input(&input_path, &json);
}

fn read_main() -> String {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: my_program <file>");
        std::process::exit(1);
    }

    args[1].clone()
}

fn write_to_input(input_path: &str, json: &serde_json::Value) {
    let path = Path::new(input_path);

    // Replace extension with .json
    let output_path = path.with_extension("json");

    let pretty = serde_json::to_string_pretty(json)
        .expect("Failed to serialize JSON");

    fs::write(&output_path, pretty)
        .expect("Failed to write JSON output");

    println!("Wrote output to {}", output_path.display());
}
