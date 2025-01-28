use std::collections::HashSet;

use crate::lexer::{Token, TokenKind};

pub struct Function<'src> {
    pub nodes: Vec<Node<'src>>,
    pub locals: HashSet<String>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum NodeKind<'src> {
    Add,
    Sub,
    Mul,
    Div,
    Num(i32),
    Eq,
    Ne,
    Lt,
    Le,
    ExprStmt,
    Assign,
    Var(&'src str),
}

#[derive(Debug, Clone)]
pub struct Node<'src> {
    pub kind: NodeKind<'src>,
    pub lhs: Option<Box<Node<'src>>>,
    pub rhs: Option<Box<Node<'src>>>,
}

pub struct Parser<'src> {
    source: &'src str,
    tokens: Vec<Token<'src>>,
    cursor: usize,
    locals: HashSet<String>,
}

impl<'src> Parser<'src> {
    pub fn new(source: &'src str, tokens: Vec<Token<'src>>) -> Self {
        Self {
            source,
            tokens,
            cursor: 0,
            locals: HashSet::new(),
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

    pub fn parse(&mut self) -> Function {
        let mut nodes = vec![];
        while !self.at_eof() {
            nodes.push(self.stmt());
        }

        Function {
            nodes,
            locals: self.locals.clone(),
        }
    }

    fn stmt(&mut self) -> Node<'src> {
        self.expr_stmt()
    }

    fn expr_stmt(&mut self) -> Node<'src> {
        let node = Node {
            kind: NodeKind::ExprStmt,
            lhs: Some(Box::new(self.expr())),
            rhs: None,
        };
        self.expect(";");
        node
    }

    fn expr(&mut self) -> Node<'src> {
        self.assign()
    }

    fn assign(&mut self) -> Node<'src> {
        let mut node = self.equality();

        if self.consume("=") {
            node = Node {
                kind: NodeKind::Assign,
                lhs: Some(Box::new(node)),
                rhs: Some(Box::new(self.assign())),
            }
        }

        node
    }

    fn equality(&mut self) -> Node<'src> {
        let mut node = self.relational();

        loop {
            if self.consume("==") {
                node = Node {
                    kind: NodeKind::Eq,
                    lhs: Some(Box::new(node)),
                    rhs: Some(Box::new(self.relational())),
                };
            } else if self.consume("!=") {
                node = Node {
                    kind: NodeKind::Ne,
                    lhs: Some(Box::new(node)),
                    rhs: Some(Box::new(self.relational())),
                };
            } else {
                return node;
            }
        }
    }

    fn relational(&mut self) -> Node<'src> {
        let mut node = self.add();

        loop {
            if self.consume("<") {
                node = Node {
                    kind: NodeKind::Lt,
                    lhs: Some(Box::new(node)),
                    rhs: Some(Box::new(self.add())),
                };
            } else if self.consume("<=") {
                node = Node {
                    kind: NodeKind::Le,
                    lhs: Some(Box::new(node)),
                    rhs: Some(Box::new(self.add())),
                };
            } else if self.consume(">") {
                node = Node {
                    kind: NodeKind::Lt,
                    lhs: Some(Box::new(self.add())),
                    rhs: Some(Box::new(node)),
                };
            } else if self.consume(">=") {
                node = Node {
                    kind: NodeKind::Le,
                    lhs: Some(Box::new(self.add())),
                    rhs: Some(Box::new(node)),
                };
            } else {
                return node;
            }
        }
    }

    fn add(&mut self) -> Node<'src> {
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

    fn mul(&mut self) -> Node<'src> {
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

    fn unary(&mut self) -> Node<'src> {
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

    fn primary(&mut self) -> Node<'src> {
        if self.consume("(") {
            let node = self.expr();
            self.expect(")");
            return node;
        }

        let token = &self.tokens[self.cursor];
        if token.kind == TokenKind::Ident {
            self.locals.insert(token.raw_str.to_string());

            self.cursor += 1;

            return Node {
                kind: NodeKind::Var(token.raw_str),
                lhs: None,
                rhs: None,
            };
        }

        Node {
            kind: NodeKind::Num(self.expect_number()),
            lhs: None,
            rhs: None,
        }
    }
}
