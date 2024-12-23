use core::panic;
use std::env::args;

use codegen::codegen;
use lexer::Lexer;
use parser::Parser;

mod codegen;
mod lexer;
mod parser;

fn main() {
    let args = args().collect::<Vec<_>>();
    if args.len() != 2 {
        panic!("引数の個数が正しくありません");
    }

    let mut lexer = Lexer::new(&args[1]);
    let tokens = lexer.lex();
    let mut parser = Parser::new(&args[1], tokens);

    let ast = parser.expr();
    codegen(ast);
}
