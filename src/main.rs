use core::panic;
use std::{env::args, io::Write};

use codegen::Codegen;
use lexer::Lexer;
use parser::Parser;

use crate::ctype::TypedObject;

mod codegen;
mod ctype;
mod lexer;
mod parser;

pub fn log(str: &str) {
    const FILE_PATH: &str = "log.txt";
    let mut f = std::fs::OpenOptions::new()
        .append(true)
        .open(FILE_PATH)
        .unwrap_or_else(|_| std::fs::File::create(FILE_PATH).unwrap());
    f.write_all(format!("{str}\n").as_bytes()).unwrap();
}

fn main() {
    let args = args().collect::<Vec<_>>();
    if args.len() != 2 {
        panic!("引数の個数が正しくありません");
    }

    let mut lexer = Lexer::new(&args[1]);
    let tokens = lexer.lex();
    let mut parser = Parser::new(&args[1], tokens);

    let functions = parser.parse();
    let typed_functions = functions
        .into_iter()
        .map(TypedObject::from)
        .collect::<Vec<_>>();

    let mut codegen = Codegen::new();
    codegen.codegen(typed_functions);
}
