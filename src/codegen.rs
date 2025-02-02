use std::collections::HashMap;

use crate::parser::{BinOps, Function, Node};

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

    pub fn codegen(&mut self, function: Function<'src>) {
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

    fn gen_addr(&self, node: Node) {
        match node {
            Node::Var(name) => {
                println!("  addi a0, fp, {}", self.locals.get(name).unwrap());
            }
            Node::Deref(node) => {
                self.gen_expr(*node);
                pop("a0");
            }
            _ => {
                panic!("{:?} is not an lvalue", node);
            }
        }
    }

    fn gen_expr(&self, node: Node) {
        match node {
            Node::Num(value) => {
                println!("  li t0, {}", value);
                push("t0");
            }
            Node::Var(_) => {
                self.gen_addr(node);
                println!("  ld t2, 0(a0)");
                push("t2");
            }
            Node::Deref(node) => {
                self.gen_expr(*node);
                pop("a0");
                println!("  ld a0, 0(a0)");
                push("a0");
            }
            Node::Addr(node) => {
                self.gen_addr(*node);
                push("a0");
            }
            Node::BinOps {
                op: BinOps::Assign,
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
            Node::BinOps { op, lhs, rhs } => {
                self.gen_expr(*lhs);
                self.gen_expr(*rhs);

                pop("t1");
                pop("t0");

                match op {
                    BinOps::Add => {
                        println!("  add t2, t0, t1");
                    }
                    BinOps::Sub => {
                        println!("  sub t2, t0, t1");
                    }
                    BinOps::Mul => {
                        println!("  mul t2, t0, t1");
                    }
                    BinOps::Div => {
                        println!("  div t2, t0, t1");
                    }
                    BinOps::Eq => {
                        println!("  xor t2, t0, t1");
                        println!("  sltiu t2, t2, 1");
                    }
                    BinOps::Ne => {
                        println!("  xor t2, t0, t1");
                        println!("  snez t2, t2");
                    }
                    BinOps::Lt => {
                        println!("  slt t2, t0, t1");
                    }
                    BinOps::Le => {
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
        match node {
            Node::For {
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
            Node::If { cond, then, els } => {
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
            Node::Block(nodes) => {
                for node in nodes {
                    self.gen_stmt(node);
                }
            }
            Node::Return(node) => {
                self.gen_expr(*node);
                println!("  j .L.return");
            }
            Node::ExprStmt(node) => {
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
