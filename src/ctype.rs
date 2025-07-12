use crate::{
    lexer::Token,
    parser::{BinOp, Node, NodeKind, Object, ObjectKind},
};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct TypedObject<'src> {
    pub name: &'src str,
    pub kind: TypedObjectKind<'src>,
}

impl<'src> From<Object<'src>> for TypedObject<'src> {
    fn from(object: Object<'src>) -> TypedObject<'src> {
        TypedObject {
            name: object.name,
            kind: object.kind.into(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum TypedObjectKind<'src> {
    Object {
        ctype: CType<'src>,
        is_local: bool,
    },
    Function {
        node: TypedNode<'src>,
        locals: Vec<TypedObject<'src>>,
        params: Vec<TypedObject<'src>>,
    },
}

impl<'src> From<ObjectKind<'src>> for TypedObjectKind<'src> {
    fn from(kind: ObjectKind<'src>) -> Self {
        match kind {
            ObjectKind::Object { ctype, is_local } => TypedObjectKind::Object { ctype, is_local },
            ObjectKind::Function {
                node,
                locals,
                params,
            } => TypedObjectKind::Function {
                node: (node).into(),
                locals: locals
                    .into_iter()
                    .map(|local| local.into())
                    .collect::<Vec<_>>(),
                params: params
                    .into_iter()
                    .map(|param| param.into())
                    .collect::<Vec<_>>(),
            },
        }
    }
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
    Var(Box<TypedObject<'src>>),
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
    Array {
        base: Box<CType<'src>>,
        len: usize,
    },
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct CType<'src> {
    pub kind: CTypeKind<'src>,
    pub name: Option<Token<'src>>,
    pub size: usize,
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

pub fn array_of<'src>(base: CType<'src>, len: usize) -> CType<'src> {
    let size = base.size;
    CType::new(
        CTypeKind::Array {
            base: Box::new(base),
            len,
        },
        None,
        size * len,
    )
}

impl<'src> From<Node<'src>> for TypedNode<'src> {
    fn from(node: Node) -> TypedNode {
        match node.kind {
            NodeKind::Num(value) => TypedNode {
                kind: TypedNodeKind::Num(value),
                ctype: Some(CType {
                    kind: CTypeKind::Int,
                    name: None,
                    size: 8,
                }),
            },
            NodeKind::Var(object) => {
                if let Object {
                    name,
                    kind: ObjectKind::Object { ctype, is_local },
                } = *object
                {
                    return TypedNode {
                        kind: TypedNodeKind::Var(Box::new(TypedObject {
                            name,
                            kind: TypedObjectKind::Object {
                                ctype: ctype.clone(),
                                is_local,
                            },
                        })),
                        ctype: Some(ctype),
                    };
                }

                panic!("{object:?} is not ObjectKind::Object")
            }
            NodeKind::BinOp {
                op: op @ (BinOp::Eq | BinOp::Ne | BinOp::Le | BinOp::Lt),
                lhs,
                rhs,
            } => {
                let lhs = (*lhs).into();
                let rhs = (*rhs).into();

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
                let lhs: TypedNode<'_> = (*lhs).into();
                let rhs: TypedNode<'_> = (*rhs).into();

                match (&op, lhs.ctype.clone(), rhs.ctype.clone()) {
                (BinOp::Assign, lhs_ctype, _) => {
                    if let CTypeKind::Array { .. }  = lhs_ctype.clone().unwrap().kind {
                        panic!("not a lvalue");
                    }

                    TypedNode {
                    kind: TypedNodeKind::BinOp {
                        op,
                        lhs: Box::new(lhs),
                        rhs: Box::new(rhs),
                    },
                    ctype: lhs_ctype.clone(),
                }},
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
                        kind: CTypeKind::Ptr(ctype) | CTypeKind::Array { base: ctype, .. },
                        ..
                    }),
                ) => {
                    let lhs = TypedNode {
                        kind: TypedNodeKind::BinOp {
                            op: BinOp::Mul,
                            lhs: Box::new(lhs),
                            rhs: Box::new((Node {
                                kind: NodeKind::Num(ctype.size as i32),
                            }).into()),
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
                        kind: CTypeKind::Ptr(ctype) | CTypeKind::Array{ base: ctype, ..},
                        ..
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
                            rhs: Box::new((Node {
                                kind: NodeKind::Num(ctype.size as i32),
                            }).into()),
                        },
                        ctype: Some(CType {
                            kind: CTypeKind::Int,
                            name: None,
                            // TODO: リテラルから変える？
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
                        kind: CTypeKind::Ptr(lhs_basety) | CTypeKind::Array { base: lhs_basety, .. },
                        ..
                    }),
                    Some(CType {
                        kind: CTypeKind::Ptr(_) | CTypeKind::Array { .. },
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
                            // TODO: リテラルから変える？
                            size: 8
                        }),
                    };

                    TypedNode {
                        kind: TypedNodeKind::BinOp {
                            op: BinOp::Div,
                            lhs: Box::new(typed_node),
                            rhs: Box::new((Node {
                                kind: NodeKind::Num(lhs_basety.size as i32),
                            }).into()),
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
                    args: args.into_iter().map(|arg| arg.into()).collect::<Vec<_>>(),
                },
                ctype: Some(CType {
                    kind: CTypeKind::Int,
                    name: None,
                    size: 8,
                }),
            },
            NodeKind::Addr(node) => {
                let typed_node: TypedNode<'_> = (*node).into();
                let ctype = match (&typed_node.ctype, &typed_node.kind) {
                    (
                        Some(CType {
                            kind: CTypeKind::Array { base, .. },
                            ..
                        }),
                        _,
                    ) => CType::pointer_to((**base).clone()),
                    (Some(ty), TypedNodeKind::Var { .. } /* | TypedNodeKind::Deref(_) */) => {
                        CType::pointer_to(ty.clone())
                    }
                    _ => panic!("invalid operand for &"),
                };

                TypedNode {
                    kind: TypedNodeKind::Addr(Box::new(typed_node)),
                    ctype: Some(ctype),
                }
            }
            NodeKind::Deref(node) => {
                let typed_node: TypedNode<'_> = (*node).into();
                if let CTypeKind::Array { base, .. } | CTypeKind::Ptr(base) =
                    &typed_node.ctype.clone().unwrap().kind
                {
                    return TypedNode {
                        kind: TypedNodeKind::Deref(Box::new(typed_node)),
                        ctype: Some((**base).clone()),
                    };
                }

                panic!("invalid pointer dereference")
            }
            NodeKind::ExprStmt(node) => {
                let typed_node = Box::new((*node).into());
                TypedNode {
                    kind: TypedNodeKind::ExprStmt(typed_node),
                    ctype: None,
                }
            }
            NodeKind::Return(node) => {
                let typed_node = Box::new((*node).into());
                TypedNode {
                    kind: TypedNodeKind::Return(typed_node),
                    ctype: None,
                }
            }
            NodeKind::Block(nodes) => {
                let typed_nodes = nodes
                    .into_iter()
                    .map(|node| node.into())
                    .collect::<Vec<_>>();
                TypedNode {
                    kind: TypedNodeKind::Block(typed_nodes),
                    ctype: None,
                }
            }
            NodeKind::If { cond, then, els } => {
                let cond = Box::new((*cond).into());
                let then = Box::new((*then).into());
                let els = els.map(|node| Box::new((*node).into()));

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
                let init = init.map(|node| Box::new((*node).into()));
                let cond = cond.map(|node| Box::new((*node).into()));
                let inc = inc.map(|node| Box::new((*node).into()));
                let then = Box::new((*then).into());

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
}
