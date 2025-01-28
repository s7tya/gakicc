use std::collections::HashSet;

use crate::lexer::{Token, TokenKind};

#[derive(Debug)]
pub struct Function<'src> {
    pub node: Node<'src>,
    pub locals: HashSet<String>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum BinOps {
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    Ne,
    Lt,
    Le,
    Assign,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Node<'src> {
    Num(i32),
    ExprStmt(Box<Node<'src>>),
    Var(&'src str),
    Return(Box<Node<'src>>),
    Block(Vec<Node<'src>>),
    If {
        cond: Box<Node<'src>>,
        then: Box<Node<'src>>,
        els: Option<Box<Node<'src>>>,
    },
    For {
        init: Option<Box<Node<'src>>>,
        cond: Option<Box<Node<'src>>>,
        inc: Option<Box<Node<'src>>>,
        then: Box<Node<'src>>,
    },
    BinOps {
        op: BinOps,
        lhs: Box<Node<'src>>,
        rhs: Box<Node<'src>>,
    },
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
        self.expect("{");

        Function {
            node: self.compound_stmt(),
            locals: self.locals.clone(),
        }
    }

    fn stmt(&mut self) -> Node<'src> {
        if self.consume("return") {
            let node = Node::Return(Box::new(self.expr()));
            self.expect(";");

            return node;
        }

        if self.consume("if") {
            self.expect("(");
            let cond = self.expr();
            self.expect(")");
            let then = self.stmt();
            let mut els = None;
            if self.consume("else") {
                els = Some(self.stmt());
            }

            return Node::If {
                cond: Box::new(cond),
                then: Box::new(then),
                els: els.map(Box::new),
            };
        }

        if self.consume("for") {
            self.expect("(");
            let init = Some(self.expr_stmt());

            let mut cond = None;
            if !self.consume(";") {
                cond = Some(self.expr());
                self.expect(";");
            }

            let mut inc = None;
            if !self.consume(")") {
                inc = Some(self.expr());
                self.expect(")");
            }

            let then = self.stmt();

            return Node::For {
                init: init.map(Box::new),
                cond: cond.map(Box::new),
                inc: inc.map(Box::new),
                then: Box::new(then),
            };
        }

        if self.consume("while") {
            self.expect("(");
            let cond = Some(self.expr());
            self.expect(")");
            let then = self.stmt();

            return Node::For {
                init: None,
                cond: cond.map(Box::new),
                inc: None,
                then: Box::new(then),
            };
        }

        if self.consume("{") {
            return self.compound_stmt();
        }

        self.expr_stmt()
    }

    fn compound_stmt(&mut self) -> Node<'src> {
        let mut nodes = vec![];
        while !self.consume("}") {
            nodes.push(self.stmt());
        }

        Node::Block(nodes)
    }

    fn expr_stmt(&mut self) -> Node<'src> {
        if self.consume(";") {
            return Node::Block(vec![]);
        }

        let node = Node::ExprStmt(Box::new(self.expr()));
        self.expect(";");
        node
    }

    fn expr(&mut self) -> Node<'src> {
        self.assign()
    }

    fn assign(&mut self) -> Node<'src> {
        let mut node = self.equality();

        if self.consume("=") {
            node = Node::BinOps {
                op: BinOps::Assign,
                lhs: Box::new(node),
                rhs: Box::new(self.assign()),
            }
        }

        node
    }

    fn equality(&mut self) -> Node<'src> {
        let mut node = self.relational();

        loop {
            if self.consume("==") {
                node = Node::BinOps {
                    op: BinOps::Eq,
                    lhs: Box::new(node),
                    rhs: Box::new(self.relational()),
                };
            } else if self.consume("!=") {
                node = Node::BinOps {
                    op: BinOps::Ne,
                    lhs: Box::new(node),
                    rhs: Box::new(self.relational()),
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
                node = Node::BinOps {
                    op: BinOps::Lt,
                    lhs: Box::new(node),
                    rhs: Box::new(self.add()),
                };
            } else if self.consume("<=") {
                node = Node::BinOps {
                    op: BinOps::Le,
                    lhs: Box::new(node),
                    rhs: Box::new(self.add()),
                };
            } else if self.consume(">") {
                node = Node::BinOps {
                    op: BinOps::Lt,
                    lhs: Box::new(self.add()),
                    rhs: Box::new(node),
                };
            } else if self.consume(">=") {
                node = Node::BinOps {
                    op: BinOps::Le,
                    lhs: Box::new(self.add()),
                    rhs: Box::new(node),
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
                node = Node::BinOps {
                    op: BinOps::Add,
                    lhs: Box::new(node),
                    rhs: Box::new(self.mul()),
                };
            } else if self.consume("-") {
                node = Node::BinOps {
                    op: BinOps::Sub,
                    lhs: Box::new(node),
                    rhs: Box::new(self.mul()),
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
                node = Node::BinOps {
                    op: BinOps::Mul,
                    lhs: Box::new(node),
                    rhs: Box::new(self.unary()),
                };
            } else if self.consume("/") {
                node = Node::BinOps {
                    op: BinOps::Div,
                    lhs: Box::new(node),
                    rhs: Box::new(self.unary()),
                };
            } else {
                return node;
            }
        }
    }

    fn unary(&mut self) -> Node<'src> {
        if self.consume("+") {
            return self.unary();
        }

        if self.consume("-") {
            return Node::BinOps {
                op: BinOps::Sub,
                lhs: Box::new(Node::Num(0)),
                rhs: Box::new(self.unary()),
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

            return Node::Var(token.raw_str);
        }

        Node::Num(self.expect_number())
    }
}
