use crate::ctype::{CType, CTypeKind, TypedObject};
use std::collections::HashMap;

use crate::{
    ctype::{TypedNode, TypedNodeKind},
    parser::BinOp,
};

pub struct Codegen<'src> {
    locals: HashMap<&'src str, i32>,
    count: usize,
    current_fn_name: Option<&'src str>,
}

const ARG_REG: &[&str] = &["a0", "a1", "a2", "a3", "a4", "a5", "a6", "a7"];

impl<'src> Codegen<'src> {
    pub fn new() -> Self {
        Self {
            locals: HashMap::new(),
            count: 0,
            current_fn_name: None,
        }
    }

    fn emit_data(&self, program: &[TypedObject<'src>]) {
        for function in program {
            if let TypedObject::Object { name, ctype, .. } = function {
                println!("  .global {name}");
                println!("  .section .data");
                println!("{name}:");

                println!("  .zero {}", ctype.size);
            }

            if let TypedObject::StringLiteral { id, string, .. } = function {
                let name = format!(".L..{id}");
                println!("  .global {name}");
                println!("  .section .data");
                println!("{name}:");
                println!("  .string \"{string}\"");
            }
        }
    }

    pub fn codegen(&mut self, functions: Vec<TypedObject<'src>>) {
        /*
            TODO: chibicc の assign_lvar_offsets だと ObjectKind::Function 相当の struct に
            そのまま stack_size を持たせているが、 Rust で ObjectKind::Function に stack_size を持たせるのが
            あんまり綺麗じゃない気がしてこの実装になっている。
        */
        let mut stack_sizes = vec![];
        for function in &functions {
            if let TypedObject::Function { locals, .. } = function {
                let mut offset = 0;

                for local in locals.iter().rev() {
                    if let TypedObject::Object { name, ctype, .. } = local {
                        offset += ctype.size;
                        self.locals.insert(name, -(offset as i32));
                    }
                }
                let stack_size = align_to(offset, 16);
                stack_sizes.push(stack_size);
            } else {
                stack_sizes.push(0);
            }
        }

        self.emit_data(&functions);

        for (function_index, function) in functions.into_iter().enumerate() {
            if let TypedObject::Function {
                name, node, params, ..
            } = function
            {
                self.current_fn_name = Some(name);

                println!("  .section .text");
                println!("  .global {name}");
                println!("{name}:");

                // Prologue
                push("ra");
                push("fp");
                println!("  mv fp, sp");
                println!("  addi sp, sp, -{}", stack_sizes[function_index]);

                for (param, reg) in params.iter().zip(ARG_REG) {
                    let offset = self.locals.get(param.name().unwrap()).unwrap();

                    if let TypedObject::Object {
                        ctype: CType { size, .. },
                        ..
                    } = param
                        && *size == 1
                    {
                        println!("  sb {reg}, {offset}(fp)");
                    } else {
                        println!("  sd {reg}, {offset}(fp)");
                    }
                }

                self.gen_stmt(node);

                // Epilogue
                println!(".L.return.{name}:");
                println!("  mv sp, fp");
                pop("fp");
                pop("ra");

                println!("  ret");
            }
        }
    }

    fn gen_addr(&self, node: TypedNode) {
        match node.kind {
            TypedNodeKind::Var(object) => match *object {
                TypedObject::Object { name, is_local, .. } => {
                    if is_local {
                        println!("  addi a0, fp, {}", self.locals.get(name).unwrap());
                    } else {
                        println!("  la a0, {name}")
                    }
                }
                TypedObject::StringLiteral { id, .. } => {
                    println!("  la a0, .L..{id}")
                }
                _ => panic!(
                    "object.kind is not TypedObjectKind::Object or TypedObjectKind::StringLiteral"
                ),
            },
            TypedNodeKind::Deref(node) => {
                self.gen_expr(*node);
            }
            _ => {
                panic!("{node:?} is not an lvalue");
            }
        }
    }

    fn gen_expr(&self, node: TypedNode) {
        let ctype = node.ctype.clone().unwrap();
        match node.kind {
            TypedNodeKind::Num(value) => {
                println!("  li a0, {value}");
            }
            TypedNodeKind::Var(_) => {
                self.gen_addr(node);
                load(&ctype);
            }
            TypedNodeKind::Deref(node) => {
                self.gen_expr(*node);
                load(&ctype);
            }
            TypedNodeKind::Addr(node) => {
                self.gen_addr(*node);
            }
            TypedNodeKind::FuncCall { name, args } => {
                let mut nargs = 0;
                for arg in args.into_iter().rev() {
                    self.gen_expr(arg);
                    push("a0");
                    nargs += 1;
                }

                for reg in ARG_REG.iter().take(nargs) {
                    pop(reg);
                }

                println!("  call {name}");
            }
            TypedNodeKind::BinOp {
                op: BinOp::Assign,
                lhs,
                rhs,
            } => {
                self.gen_addr(*lhs);
                push("a0");

                self.gen_expr(*rhs);
                store(&node.ctype.unwrap());
            }
            TypedNodeKind::BinOp { op, lhs, rhs } => {
                self.gen_expr(*lhs);
                push("a0");
                self.gen_expr(*rhs);
                push("a0");

                pop("t1");
                pop("t0");

                match op {
                    BinOp::Add => {
                        println!("  add a0, t0, t1");
                    }
                    BinOp::Sub => {
                        println!("  sub a0, t0, t1");
                    }
                    BinOp::Mul => {
                        println!("  mul a0, t0, t1");
                    }
                    BinOp::Div => {
                        println!("  div a0, t0, t1");
                    }
                    BinOp::Eq => {
                        println!("  xor a0, t0, t1");
                        println!("  sltiu a0, a0, 1");
                    }
                    BinOp::Ne => {
                        println!("  xor a0, t0, t1");
                        println!("  snez a0, a0");
                    }
                    BinOp::Lt => {
                        println!("  slt a0, t0, t1");
                    }
                    BinOp::Le => {
                        println!("  slt a0, t1, t0");
                        println!("  xori a0, a0, 1");
                    }
                    _ => unreachable!(),
                }
            }

            _ => panic!("invalid expression: {:?}", node.kind),
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
                println!("  j .L.return.{}", self.current_fn_name.unwrap());
            }
            TypedNodeKind::ExprStmt(node) => {
                self.gen_expr(*node);
            }
            _ => {
                panic!("invalid statement: {:?}", node.kind);
            }
        }
    }
}

fn align_to(n: usize, align: usize) -> usize {
    n.div_ceil(align) * align
}

fn push(reg: &str) {
    println!("  # push {reg}");
    println!("  addi sp, sp, -8");
    println!("  sd {reg}, 0(sp)");
}

fn pop(reg: &str) {
    println!("  # pop {reg}");
    println!("  ld {reg}, 0(sp)");
    println!("  addi sp, sp, 8");
}

fn load(ty: &CType) {
    if let CTypeKind::Array { .. } = ty.kind {
        return;
    }

    if ty.size == 1 {
        println!("  lb a0, 0(a0)");
    } else {
        println!("  ld a0, 0(a0)");
    }
}

fn store(ty: &CType) {
    pop("a1");

    if ty.size == 1 {
        println!("  sb a0, 0(a1)");
    } else {
        println!("  sd a0, 0(a1)");
    }
}
