use crate::parser::{BinOps, NodeKind};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum CType {
    Int,
    Ptr(Box<CType>),
    Statement,
}

impl CType {
    pub fn new(kind: &NodeKind) -> Self {
        match &kind {
            NodeKind::BinOps {
                op: BinOps::Eq | BinOps::Ne | BinOps::Le | BinOps::Lt,
                ..
            } => CType::Int,
            NodeKind::Num(_) => CType::Int,
            NodeKind::Var(_) => CType::Int,
            NodeKind::BinOps { op, lhs, rhs } => {
                match (&op, &lhs.ctype, &rhs.ctype) {
                    (BinOps::Assign, ctype, _) => ctype.clone(),
                    (_, CType::Int, CType::Int) => CType::Int,
                    // int + ptr -> ptr
                    (BinOps::Add, CType::Int, CType::Ptr(ctype)) => CType::Ptr(ctype.clone()),
                    // ptr + int, ptr - int -> ptr
                    (BinOps::Add | BinOps::Sub, CType::Ptr(ctype), CType::Int) => {
                        CType::Ptr(ctype.clone())
                    }
                    // ptr - ptr -> int
                    (BinOps::Sub, CType::Ptr(_), CType::Ptr(_)) => CType::Int,
                    (_, CType::Int, CType::Ptr(_))
                    | (_, CType::Statement, _)
                    | (_, _, CType::Statement)
                    | (_, CType::Ptr(_), CType::Int)
                    | (_, CType::Ptr(_), CType::Ptr(_)) => panic!("{:?} {:?} {:?}", lhs, op, rhs),
                }
            }
            NodeKind::Addr(node) => match node.kind {
                NodeKind::Var(_) | NodeKind::Deref(_) => CType::Ptr(Box::new(node.ctype.clone())),
                _ => panic!(),
            },
            NodeKind::Deref(node) => match &node.ctype {
                CType::Ptr(ctype) => *ctype.clone(),
                _ => CType::Int,
            },
            NodeKind::ExprStmt(..)
            | NodeKind::Return(..)
            | NodeKind::Block(..)
            | NodeKind::If { .. }
            | NodeKind::For { .. } => CType::Statement,
        }
    }
}
