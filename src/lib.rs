use backend::codegen::Generator;
use lexer::{lexer::Lexer, tokens::token_specs};
use parser::parser::Parser;
use wasm_bindgen::prelude::wasm_bindgen;

pub mod backend;
pub mod cli;
pub mod diag;
pub mod errors;
pub mod fs;
pub mod lexer;
pub mod parser;
pub mod regex;

// Allows compilation to run through web assembly bindings
#[wasm_bindgen]
pub fn compile_source(src: &str) -> String {
    let src_content = src.to_string();
    let mut dst_buf = Vec::new();
    let lexer = Lexer::new(&src_content, token_specs());
    let mut parser = Parser::new(lexer, &src_content);
    let program = parser.parse().map_err(|e| e.to_string()).unwrap();
    let mut compiler = Generator::new(program);
    println!("sigma");
    compiler
        .compile(&mut dst_buf)
        .map_err(|e| e.to_string())
        .unwrap();
    String::from_utf8(dst_buf).unwrap()
}
