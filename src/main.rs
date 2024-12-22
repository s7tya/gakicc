use core::panic;
use std::{env::args, str::FromStr};

fn main() {
    let args = args().collect::<Vec<_>>();
    if args.len() != 2 {
        panic!("引数の個数が正しくありません");
    }

    let mut sc = Scanner::new(&args[1]);

    println!("  .globl main");
    println!("main:");
    println!("  li a0, {}", sc.str_to_fromstr::<i32>().unwrap());

    while let Some(c) = sc.source.chars().nth(sc.cursor) {
        if c == '+' {
            sc.cursor += 1;
            println!("  addi a0, a0, {}", sc.str_to_fromstr::<i32>().unwrap());
            continue;
        }

        if c == '-' {
            sc.cursor += 1;
            println!("  addi a0, a0, -{}", sc.str_to_fromstr::<i32>().unwrap());
            continue;
        }

        panic!(
            "予期しない文字です: '{}'",
            sc.source.chars().nth(sc.cursor).unwrap()
        );
    }

    println!("  ret");
}

struct Scanner<'src> {
    pub source: &'src str,
    pub cursor: usize,
}

impl<'src> Scanner<'src> {
    pub fn new(source: &'src str) -> Self {
        Self { source, cursor: 0 }
    }

    pub fn str_to_fromstr<F: FromStr>(&mut self) -> Result<F, F::Err> {
        let source_chars = self.source.chars().collect::<Vec<_>>();
        let start = self.cursor;
        while self.cursor < self.source.len() && source_chars[self.cursor].is_ascii_digit() {
            self.cursor += 1;
        }

        let token = &self.source[start..self.cursor];
        token.parse()
    }
}
