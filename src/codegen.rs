use crate::parser::{Node, NodeKind};

pub fn codegen(node: Node) {
    println!("  .global main");
    println!("main:");

    gen_expr(node);

    pop("a0");
    println!("  ret");
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

fn gen_expr(node: Node) {
    if let NodeKind::Num(value) = node.kind {
        println!("  li t0, {}", value);
        push("t0");
        return;
    }

    gen_expr(*node.lhs.unwrap());
    gen_expr(*node.rhs.unwrap());

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
        NodeKind::Num(_) => unreachable!(),
    }

    push("t2");
}
