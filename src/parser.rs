use crate::{
    ctype::{CType, CTypeKind},
    lexer::{Token, TokenKind},
};

#[derive(Debug)]
pub struct Function<'src> {
    pub node: Node<'src>,
    pub locals: Vec<Obj<'src>>, // vars
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum BinOp {
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
pub struct Obj<'src> {
    pub name: &'src str,
    pub ctype: CType<'src>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum NodeKind<'src> {
    Num(i32),
    ExprStmt(Box<Node<'src>>),
    Var(Obj<'src>),
    Return(Box<Node<'src>>),
    Block(Vec<Node<'src>>),
    FuncCall(&'src str),
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
    BinOp {
        op: BinOp,
        lhs: Box<Node<'src>>,
        rhs: Box<Node<'src>>,
    },
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Node<'src> {
    pub kind: NodeKind<'src>,
}

impl<'src> Node<'src> {
    pub fn new(kind: NodeKind<'src>) -> Self {
        Self { kind }
    }
}

pub struct Parser<'src> {
    source: &'src str,
    tokens: Vec<Token<'src>>,
    cursor: usize,
    locals: Vec<Obj<'src>>,
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
        if !self.is_equal(op) {
            return false;
        }
        self.cursor += 1;
        true
    }

    pub fn is_equal(&self, op: &str) -> bool {
        let token = &self.tokens[self.cursor];
        token.raw_str == op
    }

    pub fn expect(&mut self, op: &str) {
        if !self.consume(op) {
            self.error_at(&format!("'{}' ではありません", op));
        }
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

    fn get_ident(&mut self, token: Token<'src>) -> &'src str {
        if token.kind != TokenKind::Ident {
            self.expect("expected identifier");
        }

        token.raw_str
    }

    fn declspec(&mut self) -> CType<'src> {
        self.expect("int");

        CType::new(CTypeKind::Int, None)
    }

    fn declarator(&mut self, mut ty: CType<'src>) -> CType<'src> {
        while self.consume("*") {
            ty = CType::new(CTypeKind::Ptr(Box::new(ty)), None);
        }

        if self.tokens[self.cursor].kind != TokenKind::Ident {
            self.error_at("expected a variable name");
        }

        ty.name = Some(self.tokens[self.cursor].clone());
        self.cursor += 1;
        ty
    }

    fn declaration(&mut self) -> Node<'src> {
        let basety = self.declspec();

        let mut i = 0;
        let mut cur = vec![];
        while !self.is_equal(";") {
            if i > 0 {
                self.expect(",");
            }
            i += 1;

            let ty = self.declarator(basety.clone());
            let obj = Obj {
                name: self.get_ident(ty.name.clone().unwrap()),
                ctype: ty,
            };
            // TODO: ここどっちか参照にできない？
            self.locals.push(obj.clone());

            if !self.consume("=") {
                continue;
            }

            let lhs = Node::new(NodeKind::Var(obj));
            let rhs = self.assign();
            let node = Node::new(NodeKind::BinOp {
                op: BinOp::Assign,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            });
            cur.push(Node::new(NodeKind::ExprStmt(Box::new(node))));
        }

        Node::new(NodeKind::Block(cur))
    }

    fn compound_stmt(&mut self) -> Node<'src> {
        let mut nodes = vec![];
        while !self.consume("}") {
            nodes.push(if self.is_equal("int") {
                self.declaration()
            } else {
                self.stmt()
            })
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
            node = Node::new(NodeKind::BinOp {
                op: BinOp::Assign,
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
                node = Node::new(NodeKind::BinOp {
                    op: BinOp::Eq,
                    lhs: Box::new(node),
                    rhs: Box::new(self.relational()),
                });
            } else if self.consume("!=") {
                node = Node::new(NodeKind::BinOp {
                    op: BinOp::Ne,
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
                node = Node::new(NodeKind::BinOp {
                    op: BinOp::Lt,
                    lhs: Box::new(node),
                    rhs: Box::new(self.add()),
                });
            } else if self.consume("<=") {
                node = Node::new(NodeKind::BinOp {
                    op: BinOp::Le,
                    lhs: Box::new(node),
                    rhs: Box::new(self.add()),
                });
            } else if self.consume(">") {
                node = Node::new(NodeKind::BinOp {
                    op: BinOp::Lt,
                    lhs: Box::new(self.add()),
                    rhs: Box::new(node),
                });
            } else if self.consume(">=") {
                node = Node::new(NodeKind::BinOp {
                    op: BinOp::Le,
                    lhs: Box::new(self.add()),
                    rhs: Box::new(node),
                });
            } else {
                return node;
            }
        }
    }

    fn add(&mut self) -> Node<'src> {
        let mut node = self.mul();
        loop {
            if self.consume("+") {
                node = Node::new(NodeKind::BinOp {
                    op: BinOp::Add,
                    lhs: Box::new(node),
                    rhs: Box::new(self.mul()),
                });
            } else if self.consume("-") {
                node = Node::new(NodeKind::BinOp {
                    op: BinOp::Sub,
                    lhs: Box::new(node),
                    rhs: Box::new(self.mul()),
                });
            } else {
                return node;
            }
        }
    }

    fn mul(&mut self) -> Node<'src> {
        let mut node = self.unary();
        loop {
            if self.consume("*") {
                node = Node::new(NodeKind::BinOp {
                    op: BinOp::Mul,
                    lhs: Box::new(node),
                    rhs: Box::new(self.unary()),
                });
            } else if self.consume("/") {
                node = Node::new(NodeKind::BinOp {
                    op: BinOp::Div,
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
            return Node::new(NodeKind::BinOp {
                op: BinOp::Sub,
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
            // FuncCall
            if self.tokens[self.cursor + 1].raw_str == "(" {
                // ident と "(" を消費
                self.cursor += 2;
                let node = Node::new(NodeKind::FuncCall(token.raw_str));
                self.expect(")");
                return node;
            }

            // Variable
            let Some(var) = self
                .locals
                .iter()
                .find(|local| local.name == token.raw_str)
                .cloned()
            else {
                self.error_at(&format!("undefined variable: {:?}", self.locals));
            };

            self.cursor += 1;

            return Node::new(NodeKind::Var(var));
        }

        if matches!(token.kind, TokenKind::Num(..)) {
            return Node::new(NodeKind::Num(self.expect_number()));
        }

        self.error_at("expected an expression");
    }
}
