use crate::{
    lexer::Token,
    parser::{BinOp, Member, Node, NodeKind, Object},
};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum TypedObject<'src> {
    Object {
        name: &'src str,
        ctype: CType<'src>,
        is_local: bool,
    },
    StringLiteral {
        id: usize,
        ctype: CType<'src>,
        string: String,
    },
    Function {
        name: &'src str,
        node: TypedNode<'src>,
        locals: Vec<TypedObject<'src>>,
        params: Vec<TypedObject<'src>>,
    },
}

impl<'src> TypedObject<'src> {
    pub fn name(&self) -> Option<&'src str> {
        if let TypedObject::Function { name, .. } | TypedObject::Object { name, .. } = self {
            return Some(name);
        }

        None
    }
}

impl<'src> From<Object<'src>> for TypedObject<'src> {
    fn from(kind: Object<'src>) -> Self {
        match kind {
            Object::Object {
                name,
                ctype,
                is_local,
            } => TypedObject::Object {
                name,
                ctype,
                is_local,
            },
            Object::StringLiteral { id, ctype, string } => {
                TypedObject::StringLiteral { id, ctype, string }
            }
            Object::Function {
                name,
                node,
                locals,
                params,
            } => TypedObject::Function {
                name,
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
    Member {
        member: Member<'src>,
        node: Box<TypedNode<'src>>,
    },
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum CTypeKind<'src> {
    Void,
    Int,
    Char,
    Ptr(Box<CType<'src>> /* ポイント先の型 */),
    Function {
        return_ty: Box<CType<'src>>,
        params: Vec<CType<'src>>,
    },
    Array {
        base: Box<CType<'src>>,
        len: usize,
    },
    Struct {
        members: Vec<Member<'src>>,
        is_incomplete: bool,
    },
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct CType<'src> {
    pub kind: CTypeKind<'src>,
    pub name: Option<Token>,
    pub size: usize,
    pub align: usize,
}

impl<'src> CType<'src> {
    pub fn new(kind: CTypeKind<'src>, name: Option<Token>, size: usize, align: usize) -> Self {
        CType {
            kind,
            name,
            size,
            align,
        }
    }

    pub fn pointer_to(base: CType<'src>) -> Self {
        Self {
            kind: CTypeKind::Ptr(Box::new(base)),
            name: None,
            size: 8,
            align: 8,
        }
    }

    pub fn dummy() -> CType<'src> {
        CType::new(CTypeKind::Void, None, 0, 0)
    }

    pub fn int() -> CType<'src> {
        CType::new(CTypeKind::Int, None, 4, 4)
    }

    pub fn char() -> CType<'src> {
        CType::new(CTypeKind::Char, None, 1, 1)
    }
}

pub fn array_of<'src>(base: CType<'src>, len: usize) -> CType<'src> {
    let size = base.size;
    let align = base.align;
    CType::new(
        CTypeKind::Array {
            base: Box::new(base),
            len,
        },
        None,
        size * len,
        align,
    )
}

impl<'src> From<Node<'src>> for TypedNode<'src> {
    fn from(node: Node<'src>) -> TypedNode<'src> {
        match node.kind {
            NodeKind::Num(value) => TypedNode {
                kind: TypedNodeKind::Num(value),
                ctype: Some(CType::int()),
            },
            NodeKind::Var(object) => match *object {
                Object::Object {
                    name,
                    ctype,
                    is_local,
                } => TypedNode {
                    kind: TypedNodeKind::Var(Box::new(TypedObject::Object {
                        name,
                        ctype: ctype.clone(),
                        is_local,
                    })),
                    ctype: Some(ctype),
                },
                Object::StringLiteral { id, ctype, string } => TypedNode {
                    kind: TypedNodeKind::Var(Box::new(TypedObject::StringLiteral {
                        id,
                        ctype: ctype.clone(),
                        string,
                    })),
                    ctype: Some(ctype),
                },
                _ => {
                    panic!("{object:?} is not ObjectKind::Object or ObjectKind::StringLiteral")
                }
            },
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
                    ctype: Some(CType::int()),
                }
            }
            NodeKind::BinOp {
                op: op @ BinOp::Assign,
                lhs,
                rhs,
            } => {
                let lhs: TypedNode<'_> = (*lhs).into();
                let rhs: TypedNode<'_> = (*rhs).into();

                if let CTypeKind::Array { .. } = lhs.ctype.clone().unwrap().kind {
                    panic!("not a lvalue");
                }

                let lhs_ctype = lhs.ctype.clone();

                TypedNode {
                    kind: TypedNodeKind::BinOp {
                        op,
                        lhs: Box::new(lhs),
                        rhs: Box::new(rhs),
                    },
                    ctype: lhs_ctype,
                }
            }
            NodeKind::BinOp { op, lhs, rhs } => {
                let lhs: TypedNode<'_> = (*lhs).into();
                let rhs: TypedNode<'_> = (*rhs).into();

                match (&op, lhs.ctype.as_ref(), rhs.ctype.as_ref()) {
                    // (int | char) _ (int | char) -> int
                    (
                        _,
                        Some(CType {
                            kind: CTypeKind::Int | CTypeKind::Char,
                            ..
                        }),
                        Some(CType {
                            kind: CTypeKind::Int | CTypeKind::Char,
                            ..
                        }),
                    ) => TypedNode {
                        kind: TypedNodeKind::BinOp {
                            op,
                            lhs: Box::new(lhs),
                            rhs: Box::new(rhs),
                        },
                        ctype: Some(CType::int()),
                    },
                    // (int | char) + ptr -> ptr
                    (
                        BinOp::Add,
                        Some(CType {
                            kind: CTypeKind::Int | CTypeKind::Char,
                            ..
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
                                rhs: Box::new(
                                    (Node {
                                        kind: NodeKind::Num(ctype.size as i32),
                                    })
                                    .into(),
                                ),
                            },
                            ctype: Some(CType::int()),
                        };

                        TypedNode {
                            kind: TypedNodeKind::BinOp {
                                op,
                                lhs: Box::new(lhs),
                                rhs: Box::new(rhs.clone()),
                            },
                            ctype: Some(CType::pointer_to((**ctype).clone())),
                        }
                    }
                    // ptr + (int | char), ptr - (int | char)
                    (
                        BinOp::Add | BinOp::Sub,
                        Some(CType {
                            kind: CTypeKind::Ptr(ctype) | CTypeKind::Array { base: ctype, .. },
                            ..
                        }),
                        Some(CType {
                            kind: CTypeKind::Int | CTypeKind::Char,
                            ..
                        }),
                    ) => {
                        let rhs = TypedNode {
                            kind: TypedNodeKind::BinOp {
                                op: BinOp::Mul,
                                lhs: Box::new(rhs),
                                rhs: Box::new(
                                    (Node {
                                        kind: NodeKind::Num(ctype.size as i32),
                                    })
                                    .into(),
                                ),
                            },
                            ctype: Some(CType::int()),
                        };

                        TypedNode {
                            kind: TypedNodeKind::BinOp {
                                op,
                                lhs: Box::new(lhs.clone()),
                                rhs: Box::new(rhs),
                            },
                            ctype: Some(CType::pointer_to((**ctype).clone())),
                        }
                    }
                    // ptr - ptr
                    (
                        BinOp::Sub,
                        Some(CType {
                            kind:
                                CTypeKind::Ptr(lhs_basety)
                                | CTypeKind::Array {
                                    base: lhs_basety, ..
                                },
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
                                lhs: Box::new(lhs.clone()),
                                rhs: Box::new(rhs),
                            },
                            ctype: Some(CType::int()),
                        };

                        TypedNode {
                            kind: TypedNodeKind::BinOp {
                                op: BinOp::Div,
                                lhs: Box::new(typed_node),
                                rhs: Box::new(
                                    (Node {
                                        kind: NodeKind::Num(lhs_basety.size as i32),
                                    })
                                    .into(),
                                ),
                            },
                            ctype: Some(CType::int()),
                        }
                    }
                    (BinOp::Comma, _, rhs_ty) => TypedNode {
                        kind: TypedNodeKind::BinOp {
                            op,
                            lhs: Box::new(lhs),
                            rhs: Box::new(rhs.clone()),
                        },
                        ctype: rhs_ty.cloned(),
                    },

                    // else
                    // TODO: これ本当は wildcard にしない方がいい気がする
                    (_, _, _) => {
                        panic!("{lhs:?}\n{op:?}\n{rhs:?} is not defined")
                    }
                }
            }
            NodeKind::FuncCall { name, args } => TypedNode {
                kind: TypedNodeKind::FuncCall {
                    name,
                    args: args.into_iter().map(|arg| arg.into()).collect::<Vec<_>>(),
                },
                ctype: Some(CType::int()),
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
                    (Some(ty), TypedNodeKind::Var { .. } | TypedNodeKind::Deref(_)) => {
                        CType::pointer_to(ty.clone())
                    }
                    _ => panic!(
                        "invalid operand for &: \n{:#?}\n\n{:#?}",
                        typed_node.ctype, typed_node.kind
                    ),
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
                    if base.kind == CTypeKind::Void {
                        panic!("invalid pointer dereference");
                    }

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
            NodeKind::Member { member, node } => {
                let ctype = Some(member.ty.clone());
                let node: Box<TypedNode<'src>> = Box::new((*node).into());

                TypedNode {
                    kind: TypedNodeKind::Member { member, node },
                    ctype,
                }
            }
        }
    }
}
