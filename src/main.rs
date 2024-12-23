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

enum NodeKind {
    Add,
    Sub,
    Mul,
    Div,
    Num(i32),
}

struct Node {
    kind: NodeKind,
    lhs: Option<Box<Node>>,
    rhs: Option<Box<Node>>,
}

fn main() {
    let args = args().collect::<Vec<_>>();
    if args.len() != 2 {
        panic!("引数の個数が正しくありません");
    }

    let mut lexer = Lexer::new(&args[1]);
    let tokens = lexer.lex();
    let mut parser = Parser::new(&args[1], tokens);
    let ast = parser.expr();

    println!("  .global main");
    println!("main:");

    gen(ast);

    pop("a0");
    println!("  ret");
}

fn push(reg: &str) {
    println!("  # push {}", reg);
    println!("  addi sp, sp, -4");
    println!("  sw {}, 0(sp)", reg);
}

fn pop(reg: &str) {
    println!("  # pop {}", reg);
    println!("  lw {}, 0(sp)", reg);
    println!("  addi sp, sp, 4");
}

fn gen(node: Node) {
    if let NodeKind::Num(value) = node.kind {
        println!("  li t0, {}", value);
        push("t0");
        return;
    }

    gen(*node.lhs.unwrap());
    gen(*node.rhs.unwrap());

    // pop
    pop("t1");

    // pop
    pop("t0");

    match node.kind {
        NodeKind::Add => {
            println!("  add t2, t0, t1");
        }
        NodeKind::Sub => {
            println!("  sub t2, t0, t1");
        }
        NodeKind::Mul => {
            println!("  mul t2, t0, t1");
        }
        NodeKind::Div => {
            println!("  div t2, t0, t1");
        }
        NodeKind::Num(_) => unreachable!(),
    }

    // push
    push("t2");
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

    fn expr(&mut self) -> Node {
        let mut node = self.mul();
        loop {
            if self.consume("+") {
                node = Node {
                    kind: NodeKind::Add,
                    lhs: Some(Box::new(node)),
                    rhs: Some(Box::new(self.mul())),
                };
            } else if self.consume("-") {
                node = Node {
                    kind: NodeKind::Sub,
                    lhs: Some(Box::new(node)),
                    rhs: Some(Box::new(self.mul())),
                };
            } else {
                return node;
            }
        }
    }

    fn mul(&mut self) -> Node {
        let mut node = self.unary();
        loop {
            if self.consume("*") {
                node = Node {
                    kind: NodeKind::Mul,
                    lhs: Some(Box::new(node)),
                    rhs: Some(Box::new(self.unary())),
                };
            } else if self.consume("/") {
                node = Node {
                    kind: NodeKind::Div,
                    lhs: Some(Box::new(node)),
                    rhs: Some(Box::new(self.unary())),
                };
            } else {
                return node;
            }
        }
    }

    fn unary(&mut self) -> Node {
        if self.consume("+") {
            return self.primary();
        }

        if self.consume("-") {
            return Node {
                kind: NodeKind::Sub,
                lhs: Some(Box::new(Node {
                    kind: NodeKind::Num(0),
                    lhs: None,
                    rhs: None,
                })),
                rhs: Some(Box::new(self.primary())),
            };
        }

        self.primary()
    }

    fn primary(&mut self) -> Node {
        if self.consume("(") {
            let node = self.expr();
            self.expect(")");
            return node;
        }

        Node {
            kind: NodeKind::Num(self.expect_number()),
            lhs: None,
            rhs: None,
        }
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

            if "+-*/()".contains(c) {
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
