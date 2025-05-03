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
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct TypedNode<'src> {
    pub kind: TypedNodeKind<'src>,
    pub ctype: CType<'src>,
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
    Function(Box<CType<'src>> /* 戻り値の型 */),
    Statement,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct CType<'src> {
    kind: CTypeKind<'src>,
    pub name: Option<Token<'src>>,
}

impl<'src> CType<'src> {
    pub fn new(kind: CTypeKind<'src>, name: Option<Token<'src>>) -> Self {
        CType { kind, name }
    }
}

pub fn type_functions(functions: Vec<Function>) -> Vec<TypedFunction> {
    functions
        .into_iter()
        .map(|function| TypedFunction {
            name: function.name,
            node: type_node(function.node),
            locals: function.locals,
        })
        .collect::<Vec<_>>()
}

fn type_node(node: Node) -> TypedNode {
    match node.kind {
        NodeKind::Num(value) => TypedNode {
            kind: TypedNodeKind::Num(value),
            ctype: CType {
                kind: CTypeKind::Int,
                name: None,
            },
        },
        NodeKind::Var(Obj { name, ctype }) => TypedNode {
            kind: TypedNodeKind::Var(Obj {
                name,
                ctype: ctype.clone(),
            }),
            ctype,
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
                ctype: CType {
                    kind: CTypeKind::Int,
                    name: None,
                },
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
                (
                    _,
                    CType {
                        kind: CTypeKind::Int,
                        ..
                    },
                    CType {
                        kind: CTypeKind::Int,
                        ..
                    },
                ) => TypedNode {
                    kind: TypedNodeKind::BinOp {
                        op,
                        lhs: Box::new(lhs),
                        rhs: Box::new(rhs),
                    },
                    ctype: CType {
                        kind: CTypeKind::Int,
                        name: None,
                    },
                },
                // int + ptr -> ptr
                (
                    BinOp::Add,
                    CType {
                        kind: CTypeKind::Int,
                        name: None,
                    },
                    CType {
                        kind: CTypeKind::Ptr(ctype),
                        name: None,
                    },
                ) => {
                    let lhs = TypedNode {
                        kind: TypedNodeKind::BinOp {
                            op: BinOp::Mul,
                            lhs: Box::new(lhs),
                            rhs: Box::new(type_node(Node {
                                kind: NodeKind::Num(8),
                            })),
                        },
                        ctype: CType {
                            kind: CTypeKind::Int,
                            name: None,
                        },
                    };

                    TypedNode {
                        kind: TypedNodeKind::BinOp {
                            op,
                            lhs: Box::new(lhs),
                            rhs: Box::new(rhs),
                        },
                        ctype: CType {
                            kind: CTypeKind::Ptr(ctype.clone()),
                            name: None,
                        },
                    }
                }
                // ptr + int, ptr - int
                (
                    BinOp::Add | BinOp::Sub,
                    CType {
                        kind: CTypeKind::Ptr(ctype),
                        name: None,
                    },
                    CType {
                        kind: CTypeKind::Int,
                        name: None,
                    },
                ) => {
                    let rhs = TypedNode {
                        kind: TypedNodeKind::BinOp {
                            op: BinOp::Mul,
                            lhs: Box::new(rhs),
                            rhs: Box::new(type_node(Node {
                                kind: NodeKind::Num(8),
                            })),
                        },
                        ctype: CType {
                            kind: CTypeKind::Int,
                            name: None,
                        },
                    };

                    TypedNode {
                        kind: TypedNodeKind::BinOp {
                            op,
                            lhs: Box::new(lhs),
                            rhs: Box::new(rhs),
                        },
                        ctype: CType {
                            kind: CTypeKind::Ptr(ctype.clone()),
                            name: None,
                        },
                    }
                }
                // ptr - ptr
                (
                    BinOp::Sub,
                    CType {
                        kind: CTypeKind::Ptr(_),
                        ..
                    },
                    CType {
                        kind: CTypeKind::Ptr(_),
                        ..
                    },
                ) => {
                    let typed_node = TypedNode {
                        kind: TypedNodeKind::BinOp {
                            op,
                            lhs: Box::new(lhs),
                            rhs: Box::new(rhs),
                        },
                        ctype: CType {
                            kind: CTypeKind::Int,
                            name: None,
                        },
                    };

                    TypedNode {
                        kind: TypedNodeKind::BinOp {
                            op: BinOp::Div,
                            lhs: Box::new(typed_node),
                            rhs: Box::new(type_node(Node {
                                kind: NodeKind::Num(8),
                            })),
                        },
                        ctype: CType {
                            kind: CTypeKind::Int,
                            name: None,
                        },
                    }
                }

                (
                    _,
                    CType {
                        kind: CTypeKind::Int,
                        ..
                    },
                    CType {
                        kind: CTypeKind::Ptr(_),
                        ..
                    },
                )
                | (
                    _,
                    CType {
                        kind: CTypeKind::Statement,
                        ..
                    },
                    _,
                )
                | (
                    _,
                    _,
                    CType {
                        kind: CTypeKind::Statement,
                        ..
                    },
                )
                | (
                    _,
                    CType {
                        kind: CTypeKind::Ptr(_),
                        ..
                    },
                    CType {
                        kind: CTypeKind::Int,
                        ..
                    },
                )
                | (
                    _,
                    CType {
                        kind: CTypeKind::Ptr(_),
                        ..
                    },
                    CType {
                        kind: CTypeKind::Ptr(_),
                        ..
                    },
                )
                // TODO: これ wildcard にしない方がいい気がする
                | (_, _, _) => {
                    panic!("{:?} {:?} {:?}", lhs, op, rhs)
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
            // TODO: FuncCall の型がわからない、statement であってる？
            ctype: CType {
                kind: CTypeKind::Statement,
                name: None,
            },
        },
        NodeKind::Addr(node) => {
            let typed_node = type_node(*node);
            let ctype = match typed_node.kind {
                TypedNodeKind::Var{..} /* | TypedNodeKind::Deref(_) */ => CType {
                    kind: CTypeKind::Ptr(Box::new(typed_node.ctype.clone())),
                    name: None,
                },
                _ => panic!(),
            };

            TypedNode {
                kind: TypedNodeKind::Addr(Box::new(typed_node)),
                ctype,
            }
        }
        NodeKind::Deref(node) => {
            let typed_node = type_node(*node);
            if let CTypeKind::Ptr(base) = &typed_node.ctype.kind.clone() {
                return TypedNode {
                    kind: TypedNodeKind::Deref(Box::new(typed_node)),
                    ctype: (**base).clone(),
                };
            }

            panic!("invalid pointer dereference")
        }
        NodeKind::ExprStmt(node) => {
            let typed_node = Box::new(type_node(*node));
            TypedNode {
                kind: TypedNodeKind::ExprStmt(typed_node),
                ctype: CType {
                    kind: CTypeKind::Statement,
                    name: None,
                },
            }
        }
        NodeKind::Return(node) => {
            let typed_node = Box::new(type_node(*node));
            TypedNode {
                kind: TypedNodeKind::Return(typed_node),
                ctype: CType {
                    kind: CTypeKind::Statement,
                    name: None,
                },
            }
        }
        NodeKind::Block(nodes) => {
            let typed_nodes = nodes.into_iter().map(type_node).collect::<Vec<_>>();
            TypedNode {
                kind: TypedNodeKind::Block(typed_nodes),
                ctype: CType {
                    kind: CTypeKind::Statement,
                    name: None,
                },
            }
        }
        NodeKind::If { cond, then, els } => {
            let cond = Box::new(type_node(*cond));
            let then = Box::new(type_node(*then));
            let els = els.map(|node| Box::new(type_node(*node)));

            TypedNode {
                kind: TypedNodeKind::If { cond, then, els },
                ctype: CType {
                    kind: CTypeKind::Statement,
                    name: None,
                },
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
                ctype: CType {
                    kind: CTypeKind::Statement,
                    name: None,
                },
            }
        }
    }
}
