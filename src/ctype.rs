use crate::parser::{BinOp, Function, Node, NodeKind};

#[derive(Debug)]
pub struct TypedFunction<'src> {
    pub node: TypedNode<'src>,
    pub locals: Vec<&'src str>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct TypedNode<'src> {
    pub kind: TypedNodeKind<'src>,
    pub ctype: CType,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum TypedNodeKind<'src> {
    Num(i32),
    ExprStmt(Box<TypedNode<'src>>),
    Var(&'src str),
    Return(Box<TypedNode<'src>>),
    Block(Vec<TypedNode<'src>>),
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
pub enum CType {
    Int,
    Ptr(Box<CType>),
    Statement,
}

pub fn type_function(function: Function) -> TypedFunction {
    TypedFunction {
        node: type_node(function.node),
        locals: function.locals,
    }
}

fn type_node(node: Node) -> TypedNode {
    match node.kind {
        NodeKind::Num(value) => TypedNode {
            kind: TypedNodeKind::Num(value),
            ctype: CType::Int,
        },
        NodeKind::Var(name) => TypedNode {
            kind: TypedNodeKind::Var(name),
            ctype: CType::Int,
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
                ctype: CType::Int,
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
                (_, CType::Int, CType::Int) => TypedNode {
                    kind: TypedNodeKind::BinOp {
                        op,
                        lhs: Box::new(lhs),
                        rhs: Box::new(rhs),
                    },
                    ctype: CType::Int,
                },
                // int + ptr -> ptr
                (BinOp::Add, CType::Int, CType::Ptr(ctype)) => {
                    let lhs = TypedNode {
                        kind: TypedNodeKind::BinOp {
                            op: BinOp::Mul,
                            lhs: Box::new(lhs),
                            rhs: Box::new(type_node(Node {
                                kind: NodeKind::Num(8),
                            })),
                        },
                        ctype: CType::Int,
                    };

                    TypedNode {
                        kind: TypedNodeKind::BinOp {
                            op,
                            lhs: Box::new(lhs),
                            rhs: Box::new(rhs),
                        },
                        ctype: CType::Ptr(ctype.clone()),
                    }
                }
                // ptr + int, ptr - int
                (BinOp::Add | BinOp::Sub, CType::Ptr(ctype), CType::Int) => {
                    let rhs = TypedNode {
                        kind: TypedNodeKind::BinOp {
                            op: BinOp::Mul,
                            lhs: Box::new(rhs),
                            rhs: Box::new(type_node(Node {
                                kind: NodeKind::Num(8),
                            })),
                        },
                        ctype: CType::Int,
                    };

                    TypedNode {
                        kind: TypedNodeKind::BinOp {
                            op,
                            lhs: Box::new(lhs),
                            rhs: Box::new(rhs),
                        },
                        ctype: CType::Ptr(ctype.clone()),
                    }
                }
                // ptr - ptr
                (BinOp::Sub, CType::Ptr(_), CType::Ptr(_)) => {
                    let typed_node = TypedNode {
                        kind: TypedNodeKind::BinOp {
                            op,
                            lhs: Box::new(lhs),
                            rhs: Box::new(rhs),
                        },
                        ctype: CType::Int,
                    };

                    TypedNode {
                        kind: TypedNodeKind::BinOp {
                            op: BinOp::Div,
                            lhs: Box::new(typed_node),
                            rhs: Box::new(type_node(Node {
                                kind: NodeKind::Num(8),
                            })),
                        },
                        ctype: CType::Int,
                    }
                }

                (_, CType::Int, CType::Ptr(_))
                | (_, CType::Statement, _)
                | (_, _, CType::Statement)
                | (_, CType::Ptr(_), CType::Int)
                | (_, CType::Ptr(_), CType::Ptr(_)) => panic!("{:?} {:?} {:?}", lhs, op, rhs),
            }
        }
        NodeKind::Addr(node) => {
            let typed_node = type_node(*node);
            let ctype = match typed_node.kind {
                TypedNodeKind::Var(_) | TypedNodeKind::Deref(_) => {
                    CType::Ptr(Box::new(typed_node.ctype.clone()))
                }
                _ => panic!(),
            };

            TypedNode {
                kind: TypedNodeKind::Addr(Box::new(typed_node)),
                ctype,
            }
        }
        NodeKind::Deref(node) => {
            let typed_node = type_node(*node);
            let ctype = match &typed_node.ctype {
                CType::Ptr(ctype) => *ctype.clone(),
                _ => CType::Int,
            };

            TypedNode {
                kind: TypedNodeKind::Deref(Box::new(typed_node)),
                ctype,
            }
        }
        NodeKind::ExprStmt(node) => {
            let typed_node = Box::new(type_node(*node));
            TypedNode {
                kind: TypedNodeKind::ExprStmt(typed_node),
                ctype: CType::Statement,
            }
        }
        NodeKind::Return(node) => {
            let typed_node = Box::new(type_node(*node));
            TypedNode {
                kind: TypedNodeKind::Return(typed_node),
                ctype: CType::Statement,
            }
        }
        NodeKind::Block(nodes) => {
            let typed_nodes = nodes.into_iter().map(type_node).collect::<Vec<_>>();
            TypedNode {
                kind: TypedNodeKind::Block(typed_nodes),
                ctype: CType::Statement,
            }
        }
        NodeKind::If { cond, then, els } => {
            let cond = Box::new(type_node(*cond));
            let then = Box::new(type_node(*then));
            let els = els.map(|node| Box::new(type_node(*node)));

            TypedNode {
                kind: TypedNodeKind::If { cond, then, els },
                ctype: CType::Statement,
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
                ctype: CType::Statement,
            }
        }
    }
}
