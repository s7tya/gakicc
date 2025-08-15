use std::rc::Rc;

use crate::{
    SourceMap,
    codegen::align_to,
    ctype::{CType, CTypeKind, CTypeRef, TypedNode, array_of},
    lexer::{Token, TokenKind},
};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Object<'src> {
    Object {
        name: &'src str,
        ctype: CTypeRef<'src>,
        is_local: bool,
    },
    StringLiteral {
        id: usize,
        ctype: CTypeRef<'src>,
        string: String,
    },
    Function {
        name: &'src str,
        node: Option<Node<'src>>,
        locals: Vec<Object<'src>>,
        params: Vec<Object<'src>>,
        ret_type: CTypeRef<'src>,
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
    Mod,
    Comma,
    LogAnd,
    LogOr,
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
        ret_ty: CTypeRef<'src>,
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
    Member {
        member: Member<'src>,
        node: Box<Node<'src>>,
    },
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Member<'src> {
    pub ty: CTypeRef<'src>,
    pub name: &'src str,
    pub offset: usize,
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

pub struct Tag<'src> {
    name: &'src str,
    ty: CTypeRef<'src>,
}

pub struct Parser<'src> {
    source_map: &'src SourceMap<'src>,
    tokens: Vec<Token>,
    cursor: usize,
    locals: Vec<Object<'src>>,
    pub globals: Vec<Object<'src>>,
    tags: Vec<Tag<'src>>,
    anon_gvar_count: usize,
}

impl<'src> Parser<'src> {
    pub fn new(source_map: &'src SourceMap<'src>, tokens: Vec<Token>) -> Self {
        Self {
            source_map,
            tokens,
            cursor: 0,
            locals: vec![],
            globals: vec![],
            tags: vec![],
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
        self.source_map.span_to_str(&token.span) == op
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
        let error_span = &self.tokens[self.cursor].span;
        self.source_map.error_at(error_span, message)
    }

    fn new_var(&mut self, name: &'src str, ctype: CTypeRef<'src>, is_local: bool) -> Object<'src> {
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

    fn new_string_literal(&mut self, string: String) -> Object<'src> {
        let obj = Object::StringLiteral {
            id: self.anon_gvar_count,
            ctype: array_of(CType::char(), string.len() + 1),
            string,
        };
        self.anon_gvar_count += 1;
        self.globals.push(obj.clone());

        obj
    }

    fn create_param_lvars(&mut self, ctype: CTypeRef<'src>) {
        if let CTypeKind::Function { params, .. } = &ctype.borrow().kind {
            for param in params {
                let name = self.get_ident(param.borrow().name.clone().unwrap());
                self.new_var(name, param.to_owned(), true);
            }
        }
    }

    fn find_var(&self, name: &str) -> Option<Object<'src>> {
        self.locals
            .iter()
            .chain(self.globals.iter().rev())
            .find(|obj| obj.name() == Some(name))
            .cloned()
    }

    fn is_function(&mut self) -> bool {
        if self.is_equal(";") {
            return false;
        }

        let cursor = self.cursor;
        let dummy = CType::dummy();
        let decl = self.declarator(dummy);
        let ty = &decl.borrow().kind;
        self.cursor = cursor;

        matches!(ty, CTypeKind::Function { .. })
    }

    fn is_typename(&mut self) -> bool {
        self.is_equal("void")
            || self.is_equal("int")
            || self.is_equal("char")
            || self.is_equal("struct")
            || self.is_equal("const")
    }

    pub fn parse(&mut self) -> Vec<Object<'src>> {
        while !self.at_eof() {
            let basety = self.declspec();

            if self.is_function() {
                self.function(basety);
            } else {
                self.global_variable(basety);
            }
        }

        self.globals.clone()
    }

    fn function(&mut self, basety: CTypeRef<'src>) {
        let ty = self.declarator(basety);
        let ret_ty = match &ty.borrow().kind {
            CTypeKind::Function { return_ty, .. } => Rc::clone(return_ty),
            _ => self.error_at("not a function"),
        };

        self.locals = vec![];
        let name = self.get_ident(ty.borrow().name.clone().unwrap());
        self.create_param_lvars(Rc::clone(&ty));
        let params = self.locals.clone();

        let idx = if let Some(i) = self.globals.iter().position(|g| g.name() == Some(name)) {
            i
        } else {
            self.globals.push(Object::Function {
                name,
                node: None,
                locals: vec![],
                params: params.clone(),
                ret_type: ret_ty.clone(),
            });
            self.globals.len() - 1
        };

        if self.consume(";") {
            if let Object::Function {
                ret_type,
                params: p,
                ..
            } = &mut self.globals[idx]
            {
                *ret_type = ret_ty;
                *p = params;
            }
            return;
        }

        self.expect("{");
        let body = self.compound_stmt();

        if let Object::Function {
            node,
            locals,
            params: p,
            ret_type,
            ..
        } = &mut self.globals[idx]
        {
            *node = Some(body);
            *locals = self.locals.clone();
            *p = params;
            *ret_type = ret_ty;
        }
    }

    fn global_variable(&mut self, basety: CTypeRef<'src>) {
        let mut is_first = true;

        while !self.consume(";") {
            if !is_first {
                self.expect(",");
            }
            is_first = false;

            let ty = self.declarator(Rc::clone(&basety));
            let span = &ty.borrow().name.clone().unwrap().span;
            self.new_var(self.source_map.span_to_str(span), ty, false);
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
            let init = Some({
                if self.is_typename() {
                    self.declaration()
                } else {
                    self.expr_stmt()
                }
            });

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

    fn get_ident(&mut self, token: Token) -> &'src str {
        if token.kind != TokenKind::Ident {
            self.error_at(&format!("expected identifier, got {token:?}"));
        }

        self.source_map.span_to_str(&token.span)
    }

    fn declspec(&mut self) -> CTypeRef<'src> {
        while self.is_typename() {
            if self.consume("const") {
                continue;
            }

            if self.consume("void") {
                return CType::new(CTypeKind::Void, None, 1, 1);
            }

            if self.consume("char") {
                return CType::char();
            }

            if self.consume("int") {
                return CType::int();
            }

            if self.consume("struct") {
                return self.struct_decl();
            }
        }

        self.error_at("typename expected");
    }

    fn func_params(&mut self, ty: CTypeRef<'src>) -> CTypeRef<'src> {
        let mut params = vec![];
        let mut is_head = true;
        while !self.consume(")") {
            if !is_head {
                self.expect(",");
            }
            is_head = false;

            let basety = self.declspec();
            let ty = self.declarator(basety);
            params.push(Rc::clone(&ty));
        }

        CType::new(
            CTypeKind::Function {
                return_ty: Box::new(ty),
                params,
            },
            // TODO: ここの name と size, align がこれでいいかわからない
            None,
            0,
            0,
        )
    }

    fn type_suffix(&mut self, ty: CTypeRef<'src>) -> CTypeRef<'src> {
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

    fn declarator(&mut self, mut ty: CTypeRef<'src>) -> CTypeRef<'src> {
        while self.consume("*") {
            ty = CType::pointer_to(ty);
        }

        if self.consume("(") {
            let start = self.cursor;
            self.declarator(CType::dummy());
            self.expect(")");
            ty = self.type_suffix(ty);
            let after_suffix = self.cursor;
            self.cursor = start;
            ty = self.declarator(ty);
            self.cursor = after_suffix;

            return ty;
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
        ty.borrow_mut().name = name;

        ty
    }

    fn abstract_declarator(&mut self, mut ty: CTypeRef<'src>) -> CTypeRef<'src> {
        while self.consume("*") {
            ty = CType::pointer_to(ty);
        }

        if self.consume("(") {
            let start = self.cursor;
            self.abstract_declarator(CType::dummy());
            self.expect(")");

            ty = self.type_suffix(ty);
            let after_suffix = self.cursor;
            self.cursor = start;

            ty = self.abstract_declarator(ty);
            self.cursor = after_suffix;

            return ty;
        }

        self.type_suffix(ty)
    }

    fn typename(&mut self) -> CTypeRef<'src> {
        let ty = self.declspec();
        self.abstract_declarator(ty)
    }

    fn declaration(&mut self) -> Node<'src> {
        let basety = self.declspec();

        let mut i = 0;
        let mut cur = vec![];
        while !self.consume(";") {
            if i > 0 {
                self.expect(",");
            }
            i += 1;

            let ty = self.declarator(Rc::clone(&basety));
            if let CTypeKind::Void = ty.borrow().kind {
                self.error_at("variable declared void");
            }

            let name = self.get_ident(ty.borrow().name.clone().unwrap());
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
        let mut node = self.assign();

        if self.consume(",") {
            node = Node::new(NodeKind::BinOp {
                op: BinOp::Comma,
                lhs: Box::new(node),
                rhs: Box::new(self.expr()),
            })
        }

        node
    }

    fn to_assign(&mut self, binary: Node<'src>) -> Node<'src> {
        if let Node {
            kind: NodeKind::BinOp { op, lhs, rhs },
        } = binary
        {
            let typed_lhs: TypedNode<'src> = (*lhs).clone().into();
            let obj = Box::new(self.new_var("", CType::pointer_to(typed_lhs.ctype.unwrap()), true));

            let expr1 = Node::new(NodeKind::BinOp {
                op: BinOp::Assign,
                lhs: Box::new(Node::new(NodeKind::Var(obj.clone()))),
                rhs: Box::new(Node::new(NodeKind::Addr(lhs.clone()))),
            });

            let deref_tmp = || {
                Node::new(NodeKind::Deref(Box::new(Node::new(NodeKind::Var(
                    obj.clone(),
                )))))
            };

            let expr2 = Node::new(NodeKind::BinOp {
                op: BinOp::Assign,
                lhs: Box::new(deref_tmp()),
                rhs: Box::new(Node::new(NodeKind::BinOp {
                    op,
                    lhs: Box::new(deref_tmp()),
                    rhs,
                })),
            });

            let expr3 = deref_tmp();

            return Node::new(NodeKind::BinOp {
                op: BinOp::Comma,
                lhs: Box::new(expr1),
                rhs: Box::new(Node::new(NodeKind::BinOp {
                    op: BinOp::Comma,
                    lhs: Box::new(expr2),
                    rhs: Box::new(expr3),
                })),
            });
        }

        self.error_at("not a binary");
    }

    fn assign(&mut self) -> Node<'src> {
        let mut node = self.logor();

        if self.consume("=") {
            node = Node::new(NodeKind::BinOp {
                op: BinOp::Assign,
                lhs: Box::new(node),
                rhs: Box::new(self.assign()),
            })
        }

        if self.consume("+=") {
            let rhs = Box::new(self.assign());
            return self.to_assign(Node::new(NodeKind::BinOp {
                op: BinOp::Add,
                lhs: Box::new(node),
                rhs,
            }));
        }

        if self.consume("-=") {
            let rhs = Box::new(self.assign());
            return self.to_assign(Node::new(NodeKind::BinOp {
                op: BinOp::Sub,
                lhs: Box::new(node),
                rhs,
            }));
        }

        if self.consume("*=") {
            let rhs = Box::new(self.assign());
            return self.to_assign(Node::new(NodeKind::BinOp {
                op: BinOp::Mul,
                lhs: Box::new(node),
                rhs,
            }));
        }

        if self.consume("/=") {
            let rhs = Box::new(self.assign());
            return self.to_assign(Node::new(NodeKind::BinOp {
                op: BinOp::Div,
                lhs: Box::new(node),
                rhs,
            }));
        }

        node
    }

    fn logor(&mut self) -> Node<'src> {
        let mut node = self.logand();

        loop {
            if self.consume("||") {
                node = Node::new(NodeKind::BinOp {
                    op: BinOp::LogOr,
                    lhs: Box::new(node),
                    rhs: Box::new(self.equality()),
                })
            } else {
                return node;
            }
        }
    }

    fn logand(&mut self) -> Node<'src> {
        let mut node = self.equality();

        loop {
            if self.consume("&&") {
                node = Node::new(NodeKind::BinOp {
                    op: BinOp::LogAnd,
                    lhs: Box::new(node),
                    rhs: Box::new(self.equality()),
                })
            } else {
                return node;
            }
        }
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
            } else if self.consume("%") {
                node = Node::new(NodeKind::BinOp {
                    op: BinOp::Mod,
                    lhs: Box::new(node),
                    rhs: Box::new(self.unary()),
                })
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

        if self.consume("!") {
            return Node::new(NodeKind::BinOp {
                op: BinOp::Eq,
                lhs: Box::new(Node::new(NodeKind::Num(0))),
                rhs: Box::new(self.unary()),
            });
        }

        self.postfix()
    }

    fn struct_members(&mut self) -> Vec<Member<'src>> {
        let mut members = vec![];

        while !self.consume("}") {
            let basety = self.declspec();
            let mut i = 0;

            while !self.consume(";") {
                if i != 0 {
                    self.expect(",");
                }

                let ty = self.declarator(Rc::clone(&basety));
                let name = self
                    .source_map
                    .span_to_str(&ty.borrow().name.clone().unwrap().span);
                members.push(Member {
                    ty,
                    name,
                    offset: 0, // struct_decl で更新
                });

                i += 1;
            }
        }

        members
    }

    fn struct_union_decl(&mut self) -> CTypeRef<'src> {
        let mut tag = None;
        if self.tokens[self.cursor].kind == TokenKind::Ident {
            let token = &self.tokens[self.cursor];
            self.cursor += 1;
            tag = Some(self.source_map.span_to_str(&token.span));
        }

        if let Some(tag_name) = tag
            && !self.is_equal("{")
        {
            if let Some(tag) = self.find_tag(tag_name) {
                // タグが設定されていて、すでにそれが存在し、structのメンバの定義がない場合は該当のタグを返す
                return Rc::clone(&tag.ty);
            }

            // タグが設定されている && タグが存在しない && structのメンバの定義がない場合は incomplete な定義を追加
            let ty = CType::new(
                CTypeKind::Struct {
                    members: vec![],
                    is_incomplete: true,
                },
                None,
                0,
                0,
            );

            self.push_tag(tag_name, ty.clone());
            return ty;
        }

        self.expect("{");

        let new_members = self.struct_members();
        if let Some(tag_name) = tag {
            if let Some(tag) = self.find_tag(tag_name) {
                let mut ty_mut = tag.ty.borrow_mut();
                ty_mut.kind = CTypeKind::Struct {
                    members: new_members,
                    is_incomplete: false,
                };

                return Rc::clone(&tag.ty);
            } else {
                let ty = CType::new(
                    CTypeKind::Struct {
                        members: new_members,
                        is_incomplete: false,
                    },
                    None,
                    0,
                    1,
                );
                self.push_tag(tag_name, Rc::clone(&ty));
                return ty;
            }
        }

        CType::new(
            CTypeKind::Struct {
                members: new_members,
                is_incomplete: false,
            },
            None,
            0,
            1,
        )
    }

    fn struct_decl(&mut self) -> CTypeRef<'src> {
        let ty = self.struct_union_decl();

        {
            let mut ty_mut = ty.borrow_mut();
            let CTypeKind::Struct {
                members,
                is_incomplete,
            } = &mut ty_mut.kind
            else {
                self.error_at("not a struct");
            };

            if *is_incomplete {
                // incomplete な struct は align / offset / size の計算は不要
                return ty.to_owned();
            }

            let mut offset = 0;
            let mut align = 1;
            for member in members {
                offset = align_to(offset, member.ty.borrow().align);
                member.offset = offset;
                offset += member.ty.borrow().size;

                if align < member.ty.borrow().align {
                    align = member.ty.borrow().align;
                }
            }

            ty_mut.size = align_to(offset, align);
            ty_mut.align = align;
        }

        ty
    }

    fn push_tag(&mut self, tag: &'src str, ty: CTypeRef<'src>) {
        self.tags.push(Tag { name: tag, ty });
    }

    fn find_tag(&mut self, tag: &str) -> Option<&mut Tag<'src>> {
        self.tags.iter_mut().rev().find(|t| t.name == tag)
    }

    fn get_struct_member(&mut self, ty: CTypeRef<'src>, token: &Token) -> Member<'src> {
        let raw_token = self.source_map.span_to_str(&token.span);

        if let CType {
            kind: CTypeKind::Struct { members, .. },
            ..
        } = &*ty.borrow()
        {
            for mem in members {
                if mem.name == raw_token {
                    return mem.to_owned();
                }
            }

            self.error_at(&format!("no such member; type: {ty:#?}"));
        }

        self.error_at("no such member")
    }

    fn struct_ref(&mut self, lhs: Node<'src>, cursor: usize) -> Node<'src> {
        let token = &self.tokens[cursor].clone();

        let lhs_type = TypedNode::from(lhs.clone())
            .ctype
            .map(|ty| Rc::clone(&ty))
            .unwrap();
        if !matches!(lhs_type.borrow().kind, CTypeKind::Struct { .. }) {
            self.error_at(&format!("not a struct: {:#?}", lhs_type.borrow().kind));
        }

        let member = self.get_struct_member(lhs_type, token);
        Node::new(NodeKind::Member {
            member,
            node: Box::new(lhs),
        })
    }

    fn postfix(&mut self) -> Node<'src> {
        let mut node = self.primary();

        loop {
            if self.consume("[") {
                let idx = self.expr();
                self.expect("]");

                node = Node::new(NodeKind::Deref(Box::new(Node::new(NodeKind::BinOp {
                    op: BinOp::Add,
                    lhs: Box::new(node),
                    rhs: Box::new(idx),
                }))));
                continue;
            }

            if self.consume(".") {
                node = self.struct_ref(node, self.cursor);
                self.cursor += 1;
                continue;
            }

            if self.consume("->") {
                node = Node::new(NodeKind::Deref(Box::new(node)));
                node = self.struct_ref(node, self.cursor);
                self.cursor += 1;
                continue;
            }

            if self.consume("++") {
                let one = Box::new(Node::new(NodeKind::Num(1)));
                node = Node::new(NodeKind::BinOp {
                    op: BinOp::Sub,
                    lhs: Box::new(self.to_assign(Node::new(NodeKind::BinOp {
                        op: BinOp::Add,
                        lhs: Box::new(node),
                        rhs: one.clone(),
                    }))),
                    rhs: one,
                });
                continue;
            }

            if self.consume("--") {
                let one = Box::new(Node::new(NodeKind::Num(1)));
                node = Node::new(NodeKind::BinOp {
                    op: BinOp::Add,
                    lhs: Box::new(self.to_assign(Node::new(NodeKind::BinOp {
                        op: BinOp::Sub,
                        lhs: Box::new(node),
                        rhs: one.clone(),
                    }))),
                    rhs: one,
                });
                continue;
            }

            return node;
        }
    }

    fn funcall(&mut self) -> Node<'src> {
        let name = self.source_map.span_to_str(&self.tokens[self.cursor].span);
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
        if let Some(Object::Function { ret_type, .. }) = self.find_var(name) {
            return Node::new(NodeKind::FuncCall {
                name,
                args: cur,
                ret_ty: ret_type,
            });
        }

        self.error_at(&format!("function {name} not found"));
    }

    fn primary(&mut self) -> Node<'src> {
        if self.consume("(") {
            let node = self.expr();
            self.expect(")");
            return node;
        }

        if self.is_equal("sizeof")
            && self
                .source_map
                .span_to_str(&self.tokens[self.cursor + 1].span)
                == "("
            && {
                let cursor = self.cursor;
                self.cursor += 2;
                let result = self.is_typename();
                self.cursor = cursor;
                result
            }
        {
            self.cursor += 2;
            let ty = self.typename();
            self.expect(")");

            // log(&format!("TYPE: {ty:#?}"));

            return Node::new(NodeKind::Num(ty.borrow().size as i32));
        }

        if self.consume("sizeof") {
            let node = self.unary();
            let typed_node: TypedNode<'_> = node.into();
            return Node::new(NodeKind::Num(
                typed_node.ctype.unwrap().borrow().size as i32,
            ));
        }

        let token = &self.tokens[self.cursor];
        if token.kind == TokenKind::Ident {
            // FuncCall
            if self
                .source_map
                .span_to_str(&self.tokens[self.cursor + 1].span)
                == "("
            {
                return self.funcall();
            }

            // Variable
            let raw_str = self.source_map.span_to_str(&token.span);
            let Some(var) = self.find_var(raw_str) else {
                self.error_at(&format!(
                    "undefined variable: {:?} {:?} {:?}",
                    raw_str, self.locals, self.globals
                ));
            };

            self.cursor += 1;

            return Node::new(NodeKind::Var(Box::new(var)));
        }

        if let TokenKind::String(s) = token.kind.clone() {
            self.cursor += 1;
            return Node::new(NodeKind::Var(Box::new(self.new_string_literal(s))));
        }

        if let TokenKind::Char(c) = token.kind.clone() {
            self.cursor += 1;
            let n = u32::from(c);
            return Node::new(NodeKind::Num(i32::try_from(n).unwrap()));
        }

        if matches!(token.kind, TokenKind::Num(..)) {
            return Node::new(NodeKind::Num(self.expect_number()));
        }

        self.error_at("expected an expression");
    }
}
