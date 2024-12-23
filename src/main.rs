use core::panic;
use std::env::args;

#[derive(PartialEq, Eq)]
enum TokenKind {
    Reserved,
    Num(i32),
    Eof,
}

struct Token<'src> {
    kind: TokenKind,
    raw_str: &'src str,
}

fn main() {
    let args = args().collect::<Vec<_>>();
    if args.len() != 2 {
        panic!("引数の個数が正しくありません");
    }

    let mut lexer = Lexer::new(&args[1]);
    let tokens = lexer.lex();
    let mut parser = Parser::new(&args[1], tokens);

    println!("  .globl main");
    println!("main:");
    println!("  li a0, {}", parser.expect_number());

    while !parser.at_eof() {
        if parser.consume("+") {
            println!("  addi a0, a0, {}", parser.expect_number());
            continue;
        }

        parser.expect("-");
        println!("  addi a0, a0, -{}", parser.expect_number());
    }

    println!("  ret");
}

struct Parser<'src> {
    source: &'src str,
    tokens: Vec<Token<'src>>,
    cursor: usize,
}

impl<'src> Parser<'src> {
    pub fn new(source: &'src str, tokens: Vec<Token<'src>>) -> Self {
        Self {
            source,
            tokens,
            cursor: 0,
        }
    }

    pub fn consume(&mut self, op: &str) -> bool {
        let token = &self.tokens[self.cursor];
        if token.kind != TokenKind::Reserved || token.raw_str != op {
            return false;
        }
        self.cursor += 1;

        true
    }

    pub fn expect(&mut self, op: &str) {
        let token = &self.tokens[self.cursor];
        if token.kind != TokenKind::Reserved || token.raw_str != op {
            self.error_at(&format!("'{}' ではありません", op));
        }
        self.cursor += 1;
    }

    pub fn expect_number(&mut self) -> i32 {
        let token = &self.tokens[self.cursor];
        if let TokenKind::Num(value) = token.kind {
            self.cursor += 1;
            value
        } else {
            self.error_at("数ではありません");
        }
    }

    pub fn at_eof(&self) -> bool {
        self.tokens[self.cursor].kind == TokenKind::Eof
    }

    pub fn error_at(&self, message: &str) -> ! {
        panic!(
            "{}\n{:>width$}\n{}",
            self.source,
            "^",
            message,
            width = self.cursor + 1
        );
    }
}

struct Lexer<'src> {
    source: &'src str,
    cursor: usize,
}

impl<'src> Lexer<'src> {
    pub fn new(source: &'src str) -> Self {
        Self { source, cursor: 0 }
    }

    pub fn lex(&mut self) -> Vec<Token<'src>> {
        let mut tokens = vec![];

        while self.cursor < self.source.len() {
            let c = self.source[self.cursor..].chars().next().unwrap();

            if c.is_whitespace() {
                self.cursor += 1;
                continue;
            }

            if c == '+' || c == '-' {
                tokens.push(Token {
                    kind: TokenKind::Reserved,
                    raw_str: &self.source[self.cursor..=self.cursor],
                });
                self.cursor += 1;
                continue;
            }

            if c.is_ascii_digit() {
                let start = self.cursor;
                while self.cursor < self.source.len()
                    && self.source[self.cursor..]
                        .chars()
                        .next()
                        .unwrap()
                        .is_ascii_digit()
                {
                    self.cursor += 1;
                }
                tokens.push(Token {
                    kind: TokenKind::Num(self.source[start..self.cursor].parse::<i32>().unwrap()),
                    raw_str: &self.source[start..self.cursor],
                });
                continue;
            }

            panic!("トークナイズできません");
        }

        tokens.push(Token {
            kind: TokenKind::Eof,
            raw_str: "",
        });

        tokens
    }
}
