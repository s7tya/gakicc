use crate::{
    lexer::Token,
    parser::{
        BinOp,
        Function,
        Node,
        NodeKind,
        Obj, // TODO: Obj も Typed があった方がいい？元から型ついてるけど
    },
};

#[derive(Debug)]
pub struct TypedFunction<'src> {
    pub name: &'src str,
    pub node: TypedNode<'src>,
    pub locals: Vec<Obj<'src>>,
    pub params: Vec<Obj<'src>>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct TypedNode<'src> {
    pub kind: TypedNodeKind<'src>,
    pub ctype: Option<CType<'src>>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum TypedNodeKind<'src> {
    Num(i32),
    ExprStmt(Box<TypedNode<'src>>),
    Var(Obj<'src>),
    Return(Box<TypedNode<'src>>),
    Block(Vec<TypedNode<'src>>),
    FuncCall {
        name: &'src str,
        args: Vec<TypedNode<'src>>,
    },
    Addr(Box<TypedNode<'src>>),
    Deref(Box<TypedNode<'src>>),
    If {
        cond: Box<TypedNode<'src>>,
        then: Box<TypedNode<'src>>,
        els: Option<Box<TypedNode<'src>>>,
    },
    For {
        init: Option<Box<TypedNode<'src>>>,
        cond: Option<Box<TypedNode<'src>>>,
        inc: Option<Box<TypedNode<'src>>>,
        then: Box<TypedNode<'src>>,
    },
    BinOp {
        op: BinOp,
        lhs: Box<TypedNode<'src>>,
        rhs: Box<TypedNode<'src>>,
    },
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum CTypeKind<'src> {
    Int,
    Ptr(Box<CType<'src>> /* ポイント先の型 */),
    Function {
        return_ty: Box<CType<'src>>,
        params: Vec<CType<'src>>,
    },
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct CType<'src> {
    pub kind: CTypeKind<'src>,
    pub name: Option<Token<'src>>,
    size: usize,
}

impl<'src> CType<'src> {
    pub fn new(kind: CTypeKind<'src>, name: Option<Token<'src>>, size: usize) -> Self {
        CType { kind, name, size }
    }

    pub fn pointer_to(base: CType<'src>) -> Self {
        Self {
            kind: CTypeKind::Ptr(Box::new(base)),
            name: None,
            size: 8,
        }
    }
}

pub fn type_functions(functions: Vec<Function>) -> Vec<TypedFunction> {
    functions
        .into_iter()
        .map(|function| TypedFunction {
            name: function.name,
            node: type_node(function.node),
            locals: function.locals,
            params: function.params,
        })
        .collect::<Vec<_>>()
}

fn type_node(node: Node) -> TypedNode {
    match node.kind {
        NodeKind::Num(value) => TypedNode {
            kind: TypedNodeKind::Num(value),
            ctype: Some(CType {
                kind: CTypeKind::Int,
                name: None,
                size: 8,
            }),
        },
        NodeKind::Var(Obj { name, ctype }) => TypedNode {
            kind: TypedNodeKind::Var(Obj {
                name,
                ctype: ctype.clone(),
            }),
            ctype: Some(ctype),
        },
        NodeKind::BinOp {
            op: op @ (BinOp::Eq | BinOp::Ne | BinOp::Le | BinOp::Lt),
            lhs,
            rhs,
        } => {
            let lhs = type_node(*lhs);
            let rhs = type_node(*rhs);

            TypedNode {
                kind: TypedNodeKind::BinOp {
                    op,
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                },
                ctype: Some(CType {
                    kind: CTypeKind::Int,
                    name: None,
                    size: 8,
                }),
            }
        }
        NodeKind::BinOp { op, lhs, rhs } => {
            let lhs = type_node(*lhs);
            let rhs = type_node(*rhs);

            match (&op, lhs.ctype.clone(), rhs.ctype.clone()) {
                (BinOp::Assign, lhs_ctype, _) => TypedNode {
                    kind: TypedNodeKind::BinOp {
                        op,
                        lhs: Box::new(lhs),
                        rhs: Box::new(rhs),
                    },
                    ctype: lhs_ctype.clone(),
                },
                // int _ int -> int
                (
                    _,
                    Some(CType {
                        kind: CTypeKind::Int,
                        ..
                    }),
                    Some(CType {
                        kind: CTypeKind::Int,
                        ..
                    }),
                ) => TypedNode {
                    kind: TypedNodeKind::BinOp {
                        op,
                        lhs: Box::new(lhs),
                        rhs: Box::new(rhs),
                    },
                    ctype: Some(CType {
                        kind: CTypeKind::Int,
                        name: None,
                        size: 8
                    }),
                },
                // int + ptr -> ptr
                (
                    BinOp::Add,
                    Some(CType {
                        kind: CTypeKind::Int,
                        name: None,
                        size: 8
                    }),
                    Some(CType {
                        kind: CTypeKind::Ptr(ctype),
                        name: None,
                        size: 8
                    }),
                ) => {
                    let lhs = TypedNode {
                        kind: TypedNodeKind::BinOp {
                            op: BinOp::Mul,
                            lhs: Box::new(lhs),
                            rhs: Box::new(type_node(Node {
                                kind: NodeKind::Num(8),
                            })),
                        },
                        ctype: Some(CType {
                            kind: CTypeKind::Int,
                            name: None,
                            size: 8
                        }),
                    };

                    TypedNode {
                        kind: TypedNodeKind::BinOp {
                            op,
                            lhs: Box::new(lhs),
                            rhs: Box::new(rhs),
                        },
                        ctype: Some(CType::pointer_to(*ctype.clone())),
                    }
                }
                // ptr + int, ptr - int
                (
                    BinOp::Add | BinOp::Sub,
                    Some(CType {
                        kind: CTypeKind::Ptr(ctype),
                        name: None,
                        size: 8
                    }),
                    Some(CType {
                        kind: CTypeKind::Int,
                        name: None,
                        size: 8
                    }),
                ) => {
                    let rhs = TypedNode {
                        kind: TypedNodeKind::BinOp {
                            op: BinOp::Mul,
                            lhs: Box::new(rhs),
                            rhs: Box::new(type_node(Node {
                                kind: NodeKind::Num(8),
                            })),
                        },
                        ctype: Some(CType {
                            kind: CTypeKind::Int,
                            name: None,
                            size: 8
                        }),
                    };

                    TypedNode {
                        kind: TypedNodeKind::BinOp {
                            op,
                            lhs: Box::new(lhs),
                            rhs: Box::new(rhs),
                        },
                        ctype: Some(CType::pointer_to(*ctype.clone())),
                    }
                }
                // ptr - ptr
                (
                    BinOp::Sub,
                    Some(CType {
                        kind: CTypeKind::Ptr(_),
                        ..
                    }),
                    Some(CType {
                        kind: CTypeKind::Ptr(_),
                        ..
                    }),
                ) => {
                    let typed_node = TypedNode {
                        kind: TypedNodeKind::BinOp {
                            op,
                            lhs: Box::new(lhs),
                            rhs: Box::new(rhs),
                        },
                        ctype: Some(CType {
                            kind: CTypeKind::Int,
                            name: None,
                            size: 8
                        }),
                    };

                    TypedNode {
                        kind: TypedNodeKind::BinOp {
                            op: BinOp::Div,
                            lhs: Box::new(typed_node),
                            rhs: Box::new(type_node(Node {
                                kind: NodeKind::Num(8),
                            })),
                        },
                        ctype: Some(CType {
                            kind: CTypeKind::Int,
                            name: None,
                            size: 8
                        }),
                    }
                }

                // else
                (
                    _,
                    Some(CType {
                        kind: CTypeKind::Int,
                        ..
                    }),
                    Some(CType {
                        kind: CTypeKind::Ptr(_),
                        ..
                    }),
                )
                | (
                    _,
                    None,
                    _,
                )
                | (
                    _,
                    _,
                    None,
                )
                | (
                    _,
                    Some(CType {
                        kind: CTypeKind::Ptr(_),
                        ..
                    }),
                    Some(CType {
                        kind: CTypeKind::Int,
                        ..
                    }),
                )
                | (
                    _,
                    Some(CType {
                        kind: CTypeKind::Ptr(_),
                        ..
                    }),
                    Some(CType {
                        kind: CTypeKind::Ptr(_),
                        ..
                    }),
                )
                // TODO: これ wildcard にしない方がいい気がする
                | (_, _, _) => {
                    panic!("{lhs:?}\n{op:?}\n{rhs:?}")
                }
            }
        }
        NodeKind::FuncCall { name, args } => TypedNode {
            kind: TypedNodeKind::FuncCall {
                name,
                args: args
                    .into_iter()
                    .map(|arg| type_node(arg))
                    .collect::<Vec<_>>(),
            },
            ctype: Some(CType {
                kind: CTypeKind::Int,
                name: None,
                size: 8,
            }),
        },
        NodeKind::Addr(node) => {
            let typed_node = type_node(*node);
            let ctype = match typed_node.kind {
                TypedNodeKind::Var{..} /* | TypedNodeKind::Deref(_) */ => CType::pointer_to(typed_node.ctype.clone().unwrap()),
                _ => panic!(),
            };

            TypedNode {
                kind: TypedNodeKind::Addr(Box::new(typed_node)),
                ctype: Some(ctype),
            }
        }
        NodeKind::Deref(node) => {
            let typed_node = type_node(*node);
            if let CTypeKind::Ptr(base) = &typed_node.ctype.clone().unwrap().kind {
                return TypedNode {
                    kind: TypedNodeKind::Deref(Box::new(typed_node)),
                    ctype: Some((**base).clone()),
                };
            }

            panic!("invalid pointer dereference")
        }
        NodeKind::ExprStmt(node) => {
            let typed_node = Box::new(type_node(*node));
            TypedNode {
                kind: TypedNodeKind::ExprStmt(typed_node),
                ctype: None,
            }
        }
        NodeKind::Return(node) => {
            let typed_node = Box::new(type_node(*node));
            TypedNode {
                kind: TypedNodeKind::Return(typed_node),
                ctype: None,
            }
        }
        NodeKind::Block(nodes) => {
            let typed_nodes = nodes.into_iter().map(type_node).collect::<Vec<_>>();
            TypedNode {
                kind: TypedNodeKind::Block(typed_nodes),
                ctype: None,
            }
        }
        NodeKind::If { cond, then, els } => {
            let cond = Box::new(type_node(*cond));
            let then = Box::new(type_node(*then));
            let els = els.map(|node| Box::new(type_node(*node)));

            TypedNode {
                kind: TypedNodeKind::If { cond, then, els },
                ctype: None,
            }
        }
        NodeKind::For {
            init,
            cond,
            inc,
            then,
        } => {
            let init = init.map(|node| Box::new(type_node(*node)));
            let cond = cond.map(|node| Box::new(type_node(*node)));
            let inc = inc.map(|node| Box::new(type_node(*node)));
            let then = Box::new(type_node(*then));

            TypedNode {
                kind: TypedNodeKind::For {
                    init,
                    cond,
                    inc,
                    then,
                },
                ctype: None,
            }
        }
    }
}
