use core::panic;
use std::{
    env::args,
    fs::File,
    io::{self, Read, Write},
};

use codegen::Codegen;
use lexer::Lexer;
use parser::Parser;

use crate::{ctype::TypedObject, lexer::Span};

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

pub struct SourceMap<'src> {
    pub source: &'src str,
}

impl<'src> SourceMap<'src> {
    pub fn new(source: &'src str) -> Self {
        Self { source }
    }

    pub fn span_to_str(&self, span: &Span) -> &'src str {
        // TODO: ここで範囲外の場合をハンドル
        &self.source[span.lo..span.hi]
    }
}

fn main() {
    let args = args().collect::<Vec<_>>();
    if args.len() != 2 {
        panic!("Usage: ./{} <FILENAME>", args[0]);
    }

    let mut source = String::new();
    if &args[1] == "-" {
        io::stdin().read_to_string(&mut source).unwrap();
    } else {
        let mut file =
            File::open(&args[1]).unwrap_or_else(|_| panic!("failed to open {}", &args[1]));
        file.read_to_string(&mut source).unwrap();
    }

    let mut lexer = Lexer::new(&source);
    let tokens = lexer.lex();

    let source_map = SourceMap::new(&source);
    let mut parser = Parser::new(&source_map, tokens);

    let functions = parser.parse();
    let typed_functions = functions
        .into_iter()
        .map(TypedObject::from)
        .collect::<Vec<_>>();

    let mut codegen = Codegen::new();
    codegen.codegen(typed_functions);
}
