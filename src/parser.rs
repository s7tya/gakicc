use crate::{
    ctype::CType,
    lexer::{Token, TokenKind},
};

#[derive(Debug)]
pub struct Function<'src> {
    pub node: Node<'src>,
    pub locals: Vec<&'src str>,
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
pub enum NodeKind<'src> {
    Num(i32),
    ExprStmt(Box<Node<'src>>),
    Var(&'src str),
    Return(Box<Node<'src>>),
    Block(Vec<Node<'src>>),
    Addr(Box<Node<'src>>),
    Deref(Box<Node<'src>>),
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

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Node<'src> {
    pub kind: NodeKind<'src>,
    pub ctype: CType,
}

impl<'src> Node<'src> {
    pub fn new(kind: NodeKind<'src>) -> Self {
        Node {
            ctype: CType::new(&kind),
            kind,
        }
    }
}

pub struct Parser<'src> {
    source: &'src str,
    tokens: Vec<Token<'src>>,
    cursor: usize,
    locals: Vec<&'src str>,
}

impl<'src> Parser<'src> {
    pub fn new(source: &'src str, tokens: Vec<Token<'src>>) -> Self {
        Self {
            source,
            tokens,
            cursor: 0,
            locals: vec![],
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
            let node = Node::new(NodeKind::Return(Box::new(self.expr())));
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

            return Node::new(NodeKind::If {
                cond: Box::new(cond),
                then: Box::new(then),
                els: els.map(Box::new),
            });
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

            return Node::new(NodeKind::For {
                init: init.map(Box::new),
                cond: cond.map(Box::new),
                inc: inc.map(Box::new),
                then: Box::new(then),
            });
        }

        if self.consume("while") {
            self.expect("(");
            let cond = Some(self.expr());
            self.expect(")");
            let then = self.stmt();

            return Node::new(NodeKind::For {
                init: None,
                cond: cond.map(Box::new),
                inc: None,
                then: Box::new(then),
            });
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

        Node::new(NodeKind::Block(nodes))
    }

    fn expr_stmt(&mut self) -> Node<'src> {
        if self.consume(";") {
            return Node::new(NodeKind::Block(vec![]));
        }

        let node = Node::new(NodeKind::ExprStmt(Box::new(self.expr())));
        self.expect(";");

        node
    }

    fn expr(&mut self) -> Node<'src> {
        self.assign()
    }

    fn assign(&mut self) -> Node<'src> {
        let mut node = self.equality();

        if self.consume("=") {
            node = Node::new(NodeKind::BinOps {
                op: BinOps::Assign,
                lhs: Box::new(node),
                rhs: Box::new(self.assign()),
            })
        }

        node
    }

    fn equality(&mut self) -> Node<'src> {
        let mut node = self.relational();

        loop {
            if self.consume("==") {
                node = Node::new(NodeKind::BinOps {
                    op: BinOps::Eq,
                    lhs: Box::new(node),
                    rhs: Box::new(self.relational()),
                });
            } else if self.consume("!=") {
                node = Node::new(NodeKind::BinOps {
                    op: BinOps::Ne,
                    lhs: Box::new(node),
                    rhs: Box::new(self.relational()),
                });
            } else {
                return node;
            }
        }
    }

    fn relational(&mut self) -> Node<'src> {
        let mut node = self.add();

        loop {
            if self.consume("<") {
                node = Node::new(NodeKind::BinOps {
                    op: BinOps::Lt,
                    lhs: Box::new(node),
                    rhs: Box::new(self.add()),
                });
            } else if self.consume("<=") {
                node = Node::new(NodeKind::BinOps {
                    op: BinOps::Le,
                    lhs: Box::new(node),
                    rhs: Box::new(self.add()),
                });
            } else if self.consume(">") {
                node = Node::new(NodeKind::BinOps {
                    op: BinOps::Lt,
                    lhs: Box::new(self.add()),
                    rhs: Box::new(node),
                });
            } else if self.consume(">=") {
                node = Node::new(NodeKind::BinOps {
                    op: BinOps::Le,
                    lhs: Box::new(self.add()),
                    rhs: Box::new(node),
                });
            } else {
                return node;
            }
        }
    }

    fn new_add(mut lhs: Node<'src>, mut rhs: Node<'src>) -> Node<'src> {
        match (&lhs.ctype, &rhs.ctype) {
            (CType::Int, CType::Int) => {
                return Node::new(NodeKind::BinOps {
                    op: BinOps::Add,
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                });
            }
            (CType::Int, CType::Ptr(_)) => {
                (lhs, rhs) = (rhs, lhs);
            }
            (CType::Ptr(_), CType::Int) => {}
            (CType::Ptr(_), CType::Ptr(_)) | (CType::Statement, _) | (_, CType::Statement) => {
                panic!()
            }
        }

        rhs = Node::new(NodeKind::BinOps {
            op: BinOps::Mul,
            lhs: Box::new(rhs),
            rhs: Box::new(Node::new(NodeKind::Num(8))),
        });

        Node::new(NodeKind::BinOps {
            op: BinOps::Add,
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
        })
    }

    fn new_sub(lhs: Node<'src>, rhs: Node<'src>) -> Node<'src> {
        match (&lhs.ctype, &rhs.ctype) {
            (CType::Int, CType::Int) => Node::new(NodeKind::BinOps {
                op: BinOps::Sub,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            }),
            // ptr - ptr
            (CType::Ptr(_), CType::Ptr(_)) => {
                let node = Node::new(NodeKind::BinOps {
                    op: BinOps::Sub,
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                });
                Node::new(NodeKind::BinOps {
                    op: BinOps::Div,
                    lhs: Box::new(node),
                    rhs: Box::new(Node::new(NodeKind::Num(8))),
                })
            }
            (CType::Ptr(_), CType::Int) => {
                let node = Node::new(NodeKind::BinOps {
                    op: BinOps::Mul,
                    lhs: Box::new(rhs),
                    rhs: Box::new(Node::new(NodeKind::Num(8))),
                });

                Node::new(NodeKind::BinOps {
                    op: BinOps::Sub,
                    lhs: Box::new(lhs),
                    rhs: Box::new(node),
                })
            }
            (CType::Int, CType::Ptr(_)) | (CType::Statement, _) | (_, CType::Statement) => panic!(),
        }
    }

    fn add(&mut self) -> Node<'src> {
        let mut node = self.mul();
        loop {
            if self.consume("+") {
                node = Self::new_add(node, self.mul());
            } else if self.consume("-") {
                node = Self::new_sub(node, self.mul());
            } else {
                return node;
            }
        }
    }

    fn mul(&mut self) -> Node<'src> {
        let mut node = self.unary();
        loop {
            if self.consume("*") {
                node = Node::new(NodeKind::BinOps {
                    op: BinOps::Mul,
                    lhs: Box::new(node),
                    rhs: Box::new(self.unary()),
                });
            } else if self.consume("/") {
                node = Node::new(NodeKind::BinOps {
                    op: BinOps::Div,
                    lhs: Box::new(node),
                    rhs: Box::new(self.unary()),
                });
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
            return Node::new(NodeKind::BinOps {
                op: BinOps::Sub,
                lhs: Box::new(Node::new(NodeKind::Num(0))),
                rhs: Box::new(self.unary()),
            });
        }

        if self.consume("&") {
            return Node::new(NodeKind::Addr(Box::new(self.unary())));
        }

        if self.consume("*") {
            return Node::new(NodeKind::Deref(Box::new(self.unary())));
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
            self.locals.push(token.raw_str);

            self.cursor += 1;

            return Node::new(NodeKind::Var(token.raw_str));
        }

        Node::new(NodeKind::Num(self.expect_number()))
    }
}
