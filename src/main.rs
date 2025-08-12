use core::panic;
use std::{
    env::args,
    fs::{self, File},
    io::{self, Read, Write},
};

use codegen::Codegen;
use lexer::Lexer;
use parser::Parser;

use crate::{ctype::TypedObject, lexer::Span};

mod codegen;
mod ctype;
mod escape;
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

fn print_usage(code: i32) -> ! {
    println!("Usage: gakicc [ -o <PATH> ] <FILE>");
    std::process::exit(code);
}

struct CompileOptions<'cmd> {
    input_path: &'cmd str,
    output_path: Option<&'cmd str>,
}

fn parse_args<'cmd>(args: &'cmd [String]) -> CompileOptions<'cmd> {
    let mut input_path: Option<&str> = None;
    let mut output_path: Option<&str> = None;

    for i in 0..args.len() {
        if args[i] == "--help" {
            print_usage(0);
        }

        if args[i] == "-o" {
            output_path = Some(&args[i + 1]);
            continue;
        }

        if args[i].starts_with("-o") {
            output_path = Some(&args[i][2..]);
            continue;
        }

        input_path = Some(&args[i]);
    }

    match input_path {
        Some(input_path) => CompileOptions {
            input_path,
            output_path,
        },
        None => panic!("no input files"),
    }
}

fn get_writer(path: Option<&str>) -> Box<dyn Write> {
    if let Some(path) = path
        && path != "-"
    {
        Box::new(
            fs::OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(path)
                .unwrap_or_else(|_| panic!("failed to open {}", path)),
        )
    } else {
        Box::new(io::stdout())
    }
}

fn main() {
    let args = args().collect::<Vec<_>>();

    let options = parse_args(&args);

    let mut source = String::new();
    if options.input_path == "-" {
        io::stdin().read_to_string(&mut source).unwrap();
    } else {
        let mut file = File::open(options.input_path)
            .unwrap_or_else(|_| panic!("failed to open {}", options.input_path));
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

    let out = get_writer(options.output_path);
    let mut codegen = Codegen::new(out);
    codegen.codegen(typed_functions);
}
