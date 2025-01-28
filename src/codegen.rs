use std::collections::HashMap;

use crate::parser::{Function, Node, NodeKind};

pub struct Codegen {
    locals: HashMap<String, i32>,
    count: usize,
}

impl Codegen {
    pub fn new() -> Self {
        Self {
            locals: HashMap::new(),
            count: 0,
        }
    }

    pub fn codegen(&mut self, function: Function) {
        let mut offset = 0;
        for local in function.locals {
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

    fn gen_addr(&self, node: Node) {
        if let NodeKind::Var(name) = node.kind {
            println!("  addi a0, fp, {}", self.locals.get(name).unwrap());
            return;
        }
        panic!("not an lvalue");
    }

    fn gen_expr(&self, node: Node) {
        match node.kind {
            NodeKind::Num(value) => {
                println!("  li t0, {}", value);
                push("t0");
            }
            NodeKind::Var(_) => {
                self.gen_addr(node);
                println!("  ld t2, 0(a0)");
                push("t2");
            }
            NodeKind::Assign => {
                self.gen_addr(*node.lhs.clone().unwrap());
                push("a0");

                self.gen_expr(*node.rhs.clone().unwrap());

                pop("t0");
                pop("t1");
                println!("  sd t0, 0(t1)");

                println!("  mv t2, t0");
                push("t2");
            }

            NodeKind::Add
            | NodeKind::Sub
            | NodeKind::Mul
            | NodeKind::Div
            | NodeKind::Eq
            | NodeKind::Ne
            | NodeKind::Lt
            | NodeKind::Le => {
                self.gen_expr(*node.lhs.clone().unwrap());
                self.gen_expr(*node.rhs.clone().unwrap());

                pop("t1");
                pop("t0");

                match node.kind {
                    NodeKind::Add => {
                        println!("  add t2, t0, t1");
                    }
                    NodeKind::Sub => {
                        println!("  sub t2, t0, t1");
                    }
                    NodeKind::Mul => {
                        println!("  mul t2, t0, t1");
                    }
                    NodeKind::Div => {
                        println!("  div t2, t0, t1");
                    }
                    NodeKind::Eq => {
                        println!("  xor t2, t0, t1");
                        println!("  sltiu t2, t2, 1");
                    }
                    NodeKind::Ne => {
                        println!("  xor t2, t0, t1");
                        println!("  snez t2, t2");
                    }
                    NodeKind::Lt => {
                        println!("  slt t2, t0, t1");
                    }
                    NodeKind::Le => {
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

    fn gen_stmt(&mut self, node: Node) {
        match node.kind {
            NodeKind::If { cond, then, els } => {
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
            NodeKind::Block(nodes) => {
                for node in nodes {
                    self.gen_stmt(node);
                }
            }
            NodeKind::Return => {
                self.gen_expr(*node.lhs.unwrap());
                println!("  j .L.return");
            }
            NodeKind::ExprStmt => {
                self.gen_expr(*node.lhs.unwrap());
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
