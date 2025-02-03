use std::collections::HashMap;

use crate::{
    ctype::{TypedFunction, TypedNode, TypedNodeKind},
    parser::BinOp,
};

pub struct Codegen<'src> {
    locals: HashMap<&'src str, i32>,
    count: usize,
}

impl<'src> Codegen<'src> {
    pub fn new() -> Self {
        Self {
            locals: HashMap::new(),
            count: 0,
        }
    }

    pub fn codegen(&mut self, function: TypedFunction<'src>) {
        let mut offset = 0;

        for local in function.locals.into_iter().rev() {
            offset += 8;
            self.locals.insert(local, -(offset as i32));
        }
        let stack_size = align_to(offset, 16);

        println!("  .global main");
        println!("main:");

        // Prologue
        push("fp");
        println!("  mv fp, sp");
        println!("  addi sp, sp, -{}", stack_size);

        self.gen_stmt(function.node);

        // Epilogue
        println!(".L.return:");
        pop("a0");
        println!("  mv sp, fp");
        pop("fp");

        println!("  ret");
    }

    fn gen_addr(&self, node: TypedNode) {
        match node.kind {
            TypedNodeKind::Var(name) => {
                println!("  addi a0, fp, {}", self.locals.get(name).unwrap());
            }
            TypedNodeKind::Deref(node) => {
                self.gen_expr(*node);
                pop("a0");
            }
            _ => {
                panic!("{:?} is not an lvalue", node);
            }
        }
    }

    fn gen_expr(&self, node: TypedNode) {
        match node.kind {
            TypedNodeKind::Num(value) => {
                println!("  li t0, {}", value);
                push("t0");
            }
            TypedNodeKind::Var(_) => {
                self.gen_addr(node);
                println!("  ld t2, 0(a0)");
                push("t2");
            }
            TypedNodeKind::Deref(node) => {
                self.gen_expr(*node);
                pop("a0");
                println!("  ld a0, 0(a0)");
                push("a0");
            }
            TypedNodeKind::Addr(node) => {
                self.gen_addr(*node);
                push("a0");
            }
            TypedNodeKind::BinOp {
                op: BinOp::Assign,
                lhs,
                rhs,
            } => {
                self.gen_addr(*lhs);
                push("a0");

                self.gen_expr(*rhs);

                pop("t0");
                pop("t1");
                println!("  sd t0, 0(t1)");

                println!("  mv t2, t0");
                push("t2");
            }
            TypedNodeKind::BinOp { op, lhs, rhs } => {
                self.gen_expr(*lhs);
                self.gen_expr(*rhs);

                pop("t1");
                pop("t0");

                match op {
                    BinOp::Add => {
                        println!("  add t2, t0, t1");
                    }
                    BinOp::Sub => {
                        println!("  sub t2, t0, t1");
                    }
                    BinOp::Mul => {
                        println!("  mul t2, t0, t1");
                    }
                    BinOp::Div => {
                        println!("  div t2, t0, t1");
                    }
                    BinOp::Eq => {
                        println!("  xor t2, t0, t1");
                        println!("  sltiu t2, t2, 1");
                    }
                    BinOp::Ne => {
                        println!("  xor t2, t0, t1");
                        println!("  snez t2, t2");
                    }
                    BinOp::Lt => {
                        println!("  slt t2, t0, t1");
                    }
                    BinOp::Le => {
                        println!("  slt t2, t1, t0");
                        println!("  xori t2, t2, 1");
                    }
                    _ => unreachable!(),
                }

                push("t2");
            }

            _ => panic!("invalid expression"),
        }
    }

    fn gen_stmt(&mut self, node: TypedNode) {
        match node.kind {
            TypedNodeKind::For {
                init,
                cond,
                inc,
                then,
            } => {
                self.count += 1;
                if let Some(init) = init {
                    self.gen_stmt(*init);
                }
                println!(".L.begin.{}:", self.count);
                if let Some(cond) = cond {
                    self.gen_expr(*cond);
                    pop("a0");
                    println!("  beq a0, zero, .L.end.{}", self.count);
                }
                self.gen_stmt(*then);
                if let Some(inc) = inc {
                    self.gen_expr(*inc);
                }
                println!("  j .L.begin.{}", self.count);
                println!(".L.end.{}:", self.count);
            }
            TypedNodeKind::If { cond, then, els } => {
                self.count += 1;

                self.gen_expr(*cond);
                pop("a0");
                println!("  beq a0, zero, .L.else.{}", self.count);

                self.gen_stmt(*then);
                println!("  j .L.end.{}", self.count);
                println!(".L.else.{}:", self.count);
                if let Some(els) = els {
                    self.gen_stmt(*els);
                }
                println!(".L.end.{}:", self.count);
            }
            TypedNodeKind::Block(nodes) => {
                for node in nodes {
                    self.gen_stmt(node);
                }
            }
            TypedNodeKind::Return(node) => {
                self.gen_expr(*node);
                println!("  j .L.return");
            }
            TypedNodeKind::ExprStmt(node) => {
                self.gen_expr(*node);
                pop("zero");
            }
            _ => {
                panic!("invalid statement");
            }
        }
    }
}

fn align_to(n: usize, align: usize) -> usize {
    n.div_ceil(align) * align
}

fn push(reg: &str) {
    println!("  # push {}", reg);
    println!("  addi sp, sp, -8");
    println!("  sd {}, 0(sp)", reg);
}

fn pop(reg: &str) {
    println!("  # pop {}", reg);
    println!("  ld {}, 0(sp)", reg);
    println!("  addi sp, sp, 8");
}
