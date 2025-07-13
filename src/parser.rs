use crate::{
    ctype::{CType, CTypeKind, TypedNode, array_of},
    lexer::{Token, TokenKind},
};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Object<'src> {
    Object {
        name: &'src str,
        ctype: CType<'src>,
        is_local: bool,
    },
    StringLiteral {
        id: usize,
        ctype: CType<'src>,
        string: &'src str,
    },
    Function {
        name: &'src str,
        node: Node<'src>,
        locals: Vec<Object<'src>>,
        params: Vec<Object<'src>>,
    },
}

impl<'src> Object<'src> {
    pub fn name(&self) -> Option<&'src str> {
        if let Object::Function { name, .. } | Object::Object { name, .. } = self {
            return Some(name);
        }

        None
    }
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
pub enum NodeKind<'src> {
    Num(i32),
    ExprStmt(Box<Node<'src>>),
    Var(Box<Object<'src>>),
    Return(Box<Node<'src>>),
    Block(Vec<Node<'src>>),
    FuncCall {
        name: &'src str,
        args: Vec<Node<'src>>,
    },
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
    locals: Vec<Object<'src>>,
    pub globals: Vec<Object<'src>>,
    anon_gvar_count: usize,
}

impl<'src> Parser<'src> {
    pub fn new(source: &'src str, tokens: Vec<Token<'src>>) -> Self {
        Self {
            source,
            tokens,
            cursor: 0,
            locals: vec![],
            globals: vec![],
            anon_gvar_count: 0,
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
            self.error_at(&format!("'{op}' ではありません"));
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
            // TODO: この表示が正しくない。white spaceをスキップしている分と、文字数とトークン数の誤解
            width = self.cursor
        );
    }

    fn new_var(&mut self, name: &'src str, ctype: CType<'src>, is_local: bool) -> Object<'src> {
        let obj = Object::Object {
            name,
            ctype,
            is_local,
        };

        // TODO: ここどっちか参照にできない？
        if is_local {
            self.locals.push(obj.clone());
        } else {
            self.globals.push(obj.clone());
        }

        obj
    }

    fn new_string_literal(&mut self, string: &'src str) -> Object<'src> {
        let obj = Object::StringLiteral {
            id: self.anon_gvar_count,
            ctype: array_of(
                CType {
                    kind: CTypeKind::Char,
                    size: 1,
                    name: None,
                },
                string.len() + 1,
            ),
            string,
        };
        self.anon_gvar_count += 1;
        self.globals.push(obj.clone());

        obj
    }

    fn create_param_lvars(&mut self, ctype: CType<'src>) {
        if let CTypeKind::Function { params, .. } = ctype.kind {
            for param in params {
                let name = self.get_ident(param.name.clone().unwrap());
                self.new_var(name, param, true);
            }
        }
    }

    fn find_var(&self, name: &str) -> Option<Object<'src>> {
        self.locals
            .iter()
            .chain(&self.globals)
            .find(|obj| obj.name() == Some(name))
            .cloned()
    }

    fn is_function(&mut self) -> bool {
        if self.is_equal(";") {
            return false;
        }

        let cursor = self.cursor;
        let dummy = CType {
            name: None,
            size: 0,
            kind: CTypeKind::Int,
        };
        let ty = self.declarator(dummy).kind;
        self.cursor = cursor;

        matches!(ty, CTypeKind::Function { .. })
    }

    fn is_typename(&mut self) -> bool {
        self.is_equal("int") || self.is_equal("char")
    }

    pub fn parse(&mut self) -> Vec<Object> {
        while !self.at_eof() {
            let basety = self.declspec();

            if self.is_function() {
                let function = self.function(basety);
                self.globals.push(function);
            } else {
                self.global_variable(basety);
            }
        }

        self.globals.clone()
    }

    fn function(&mut self, basety: CType<'src>) -> Object<'src> {
        let ty = self.declarator(basety);

        self.locals = vec![];

        let name = self.get_ident(ty.name.clone().unwrap());
        self.create_param_lvars(ty);
        let params = self.locals.clone();
        self.expect("{");

        Object::Function {
            name,
            node: self.compound_stmt(),
            locals: self.locals.clone(),
            params,
        }
    }

    fn global_variable(&mut self, basety: CType<'src>) {
        let mut is_first = true;

        while !self.consume(";") {
            if !is_first {
                self.expect(",");
            }
            is_first = false;

            let ty = self.declarator(basety.clone());
            self.new_var(ty.name.clone().unwrap().raw_str, ty, false);
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
            self.error_at(&format!("expected identifier, got {token:?}"));
        }

        token.raw_str
    }

    fn declspec(&mut self) -> CType<'src> {
        if self.consume("char") {
            return CType::new(CTypeKind::Char, None, 1);
        }

        self.expect("int");

        CType::new(CTypeKind::Int, None, 8)
    }

    fn func_params(&mut self, ty: CType<'src>) -> CType<'src> {
        let mut params = vec![];
        let mut is_head = true;
        while !self.consume(")") {
            if !is_head {
                self.expect(",");
            }
            is_head = false;

            let basety = self.declspec();
            let ty = self.declarator(basety);
            params.push(ty.clone());
        }

        CType::new(
            CTypeKind::Function {
                return_ty: Box::new(ty),
                params,
            },
            // TODO: ここの name と size がこれでいいかわからない
            None,
            0,
        )
    }

    fn type_suffix(&mut self, ty: CType<'src>) -> CType<'src> {
        if self.consume("(") {
            return self.func_params(ty);
        }

        if self.consume("[") {
            let sz = self.expect_number();
            self.expect("]");
            let ty = self.type_suffix(ty);
            return array_of(ty, sz as usize);
        }

        ty
    }

    fn declarator(&mut self, mut ty: CType<'src>) -> CType<'src> {
        while self.consume("*") {
            ty = CType::pointer_to(ty);
        }

        if self.tokens[self.cursor].kind != TokenKind::Ident {
            self.error_at("expected a variable name");
        }

        // ident から名前を取得
        let name = Some(self.tokens[self.cursor].clone());
        // ident 分カーソルを進める
        self.cursor += 1;

        // その後に "(" ")" が続いた場合に型を関数に変更
        ty = self.type_suffix(ty);
        // 名前を設定
        ty.name = name;

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
            let name = self.get_ident(ty.name.clone().unwrap());
            let obj = self.new_var(name, ty, true);

            if !self.consume("=") {
                continue;
            }

            let lhs = Node::new(NodeKind::Var(Box::new(obj)));
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
            nodes.push(if self.is_typename() {
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
        if self.consume("sizeof") {
            let node = self.unary();
            let typed_node: TypedNode<'_> = node.into();
            return Node::new(NodeKind::Num(typed_node.ctype.unwrap().size as i32));
        }

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

        self.postfix()
    }

    fn postfix(&mut self) -> Node<'src> {
        let mut node = self.primary();

        while self.consume("[") {
            let idx = self.expr();
            self.expect("]");
            node = Node::new(NodeKind::Deref(Box::new(Node::new(NodeKind::BinOp {
                op: BinOp::Add,
                lhs: Box::new(node),
                rhs: Box::new(idx),
            }))))
        }

        node
    }

    fn funcall(&mut self) -> Node<'src> {
        let name = self.tokens[self.cursor].raw_str;
        // ident と "(" を消費
        self.cursor += 2;

        let mut i = 0;
        let mut cur = vec![];
        while !self.consume(")") {
            if i > 0 {
                self.expect(",");
            }
            i += 1;

            cur.push(self.assign());
        }

        Node::new(NodeKind::FuncCall { name, args: cur })
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
                return self.funcall();
            }

            // Variable
            let Some(var) = self.find_var(token.raw_str) else {
                self.error_at(&format!(
                    "undefined variable: {:?} {:?} {:?}",
                    token.raw_str, self.locals, self.globals
                ));
            };

            self.cursor += 1;

            return Node::new(NodeKind::Var(Box::new(var)));
        }

        if let TokenKind::String(s) = token.kind {
            self.cursor += 1;
            return Node::new(NodeKind::Var(Box::new(self.new_string_literal(s))));
        }

        if matches!(token.kind, TokenKind::Num(..)) {
            return Node::new(NodeKind::Num(self.expect_number()));
        }

        self.error_at("expected an expression");
    }
}
