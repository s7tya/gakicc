use crate::{
    ctype::{CTypeKind, CTypeRef, TypedObject},
    escape::escape,
};
use std::{collections::HashMap, io::Write};

use crate::{
    ctype::{TypedNode, TypedNodeKind},
    parser::BinOp,
};

pub struct Codegen<'src> {
    locals: HashMap<&'src str, i32>,
    count: usize,
    current_fn_name: Option<&'src str>,
    writer: Box<dyn Write>,
}

const ARG_REG: &[&str] = &["a0", "a1", "a2", "a3", "a4", "a5", "a6", "a7"];

impl<'src> Codegen<'src> {
    pub fn new(writer: Box<dyn Write>) -> Self {
        Self {
            locals: HashMap::new(),
            count: 0,
            current_fn_name: None,
            writer,
        }
    }

    fn emit_data(&mut self, program: &[TypedObject<'src>]) {
        for function in program {
            if let TypedObject::Object { name, ctype, .. } = function {
                writeln!(&mut self.writer, "  .global {name}").unwrap();
                writeln!(&mut self.writer, "  .section .data").unwrap();
                writeln!(&mut self.writer, "{name}:").unwrap();

                writeln!(&mut self.writer, "  .zero {}", ctype.borrow().size).unwrap();
            }

            if let TypedObject::StringLiteral { id, string, .. } = function {
                let name = format!(".L..{id}");
                writeln!(&mut self.writer, "  .global {name}").unwrap();
                writeln!(&mut self.writer, "  .section .data").unwrap();
                writeln!(&mut self.writer, "{name}:").unwrap();
                writeln!(&mut self.writer, "  .string \"{}\"", escape(string)).unwrap();
            }
        }
    }

    pub fn codegen(&mut self, functions: Vec<TypedObject<'src>>) {
        /*
            TODO: chibicc の assign_lvar_offsets だと ObjectKind::Function 相当の struct に
            そのまま stack_size を持たせているが、 Rust で ObjectKind::Function に stack_size を持たせるのが
            あんまり綺麗じゃない気がしてこの実装になっている。
        */

        self.emit_data(&functions);

        for function in functions {
            if let TypedObject::Function {
                name,
                node,
                params,
                locals,
            } = function
            {
                let mut offset = 0;
                for local in locals.iter().rev() {
                    if let TypedObject::Object { name, ctype, .. } = local {
                        let ty = ctype.borrow();
                        offset = align_to(offset, ty.align);
                        offset += ty.size;
                        self.locals.insert(name, -(offset as i32));
                    }
                }
                let stack_size = align_to(offset, 16);

                self.current_fn_name = Some(name);

                writeln!(&mut self.writer, "  .section .text").unwrap();
                writeln!(&mut self.writer, "  .global {name}").unwrap();
                writeln!(&mut self.writer, "{name}:").unwrap();

                // Prologue
                push(&mut self.writer, "ra");
                push(&mut self.writer, "fp");
                writeln!(&mut self.writer, "  mv fp, sp").unwrap();

                // RISC-V における即値の範囲は [-2048, 2047] なので、それを超える場合には addi をその分繰り返す
                addi(&mut self.writer, "sp", "sp", 0 - (stack_size as i32));

                for (param, reg) in params.iter().zip(ARG_REG) {
                    let offset = self.locals.get(param.name().unwrap()).unwrap();

                    if let TypedObject::Object { ctype, .. } = param {
                        let size = ctype.borrow().size;
                        match size {
                            1 => {
                                writeln!(&mut self.writer, "  sb {reg}, {offset}(fp)").unwrap();
                            }
                            4 => {
                                writeln!(&mut self.writer, "  sw {reg}, {offset}(fp)").unwrap();
                            }
                            _ => {
                                writeln!(&mut self.writer, "  sd {reg}, {offset}(fp)").unwrap();
                            }
                        }
                    }
                }

                self.gen_stmt(node);

                // Epilogue
                writeln!(&mut self.writer, ".L.return.{name}:").unwrap();
                writeln!(&mut self.writer, "  mv sp, fp").unwrap();
                pop(&mut self.writer, "fp");
                pop(&mut self.writer, "ra");

                writeln!(&mut self.writer, "  ret").unwrap();

                self.locals.clear();
            }
        }
    }

    fn gen_addr(&mut self, node: TypedNode) {
        match node.kind {
            TypedNodeKind::Var(object) => match *object {
                TypedObject::Object { name, is_local, .. } => {
                    if is_local {
                        addi(
                            &mut self.writer,
                            "a0",
                            "fp",
                            *self.locals.get(name).unwrap(),
                        );
                    } else {
                        writeln!(&mut self.writer, "  la a0, {name}").unwrap();
                    }
                }
                TypedObject::StringLiteral { id, .. } => {
                    writeln!(&mut self.writer, "  la a0, .L..{id}").unwrap();
                }
                _ => panic!(
                    "object.kind is not TypedObjectKind::Object or TypedObjectKind::StringLiteral"
                ),
            },
            TypedNodeKind::Deref(node) => {
                self.gen_expr(*node);
            }
            TypedNodeKind::BinOp {
                op: BinOp::Comma,
                lhs,
                rhs,
            } => {
                self.gen_expr(*lhs);
                self.gen_addr(*rhs);
            }
            TypedNodeKind::Member { node, member } => {
                self.gen_addr(*node);
                writeln!(self.writer, "  addi a0, a0, {}", member.offset).unwrap();
            }
            _ => {
                panic!("{node:?} is not an lvalue");
            }
        }
    }

    fn gen_expr(&mut self, node: TypedNode) {
        let ctype = node.ctype.clone().unwrap();
        match node.kind {
            TypedNodeKind::Num(value) => {
                writeln!(&mut self.writer, "  li a0, {value}").unwrap();
            }
            TypedNodeKind::Var(_) | TypedNodeKind::Member { .. } => {
                self.gen_addr(node);
                load(&mut self.writer, &ctype);
            }
            TypedNodeKind::Deref(node) => {
                self.gen_expr(*node);
                load(&mut self.writer, &ctype);
            }
            TypedNodeKind::Addr(node) => {
                self.gen_addr(*node);
            }
            TypedNodeKind::FuncCall { name, args } => {
                let mut nargs = 0;
                for arg in args.into_iter().rev() {
                    self.gen_expr(arg);
                    push(&mut self.writer, "a0");
                    nargs += 1;
                }

                for reg in ARG_REG.iter().take(nargs) {
                    pop(&mut self.writer, reg);
                }

                writeln!(&mut self.writer, "  call {name}").unwrap();
            }
            TypedNodeKind::BinOp {
                op: BinOp::Assign,
                lhs,
                rhs,
            } => {
                self.gen_addr(*lhs);
                push(&mut self.writer, "a0");

                self.gen_expr(*rhs);
                store(&mut self.writer, &node.ctype.unwrap());
            }
            TypedNodeKind::BinOp { op, lhs, rhs } => {
                self.gen_expr(*lhs.clone());
                push(&mut self.writer, "a0");
                self.gen_expr(*rhs.clone());
                push(&mut self.writer, "a0");

                pop(&mut self.writer, "t1");
                pop(&mut self.writer, "t0");

                let postfix = match (
                    lhs.ctype.map(|ty| ty.borrow().size).unwrap(),
                    rhs.ctype.map(|ty| ty.borrow().size).unwrap(),
                ) {
                    (4, 4) => "w",
                    (_, _) => "",
                };

                match op {
                    BinOp::Add => {
                        writeln!(&mut self.writer, "  add{postfix} a0, t0, t1",).unwrap();
                    }
                    BinOp::Sub => {
                        writeln!(&mut self.writer, "  sub{postfix} a0, t0, t1").unwrap();
                    }
                    BinOp::Mul => {
                        writeln!(&mut self.writer, "  mul{postfix} a0, t0, t1").unwrap();
                    }
                    BinOp::Div => {
                        writeln!(&mut self.writer, "  div{postfix} a0, t0, t1").unwrap();
                    }
                    BinOp::Mod => {
                        writeln!(&mut self.writer, "  rem{postfix} a0, t0, t1").unwrap();
                    }
                    BinOp::Eq => {
                        writeln!(&mut self.writer, "  xor a0, t0, t1").unwrap();
                        writeln!(&mut self.writer, "  sltiu a0, a0, 1").unwrap();
                    }
                    BinOp::Ne => {
                        writeln!(&mut self.writer, "  xor a0, t0, t1").unwrap();
                        writeln!(&mut self.writer, "  snez a0, a0").unwrap();
                    }
                    BinOp::Lt => {
                        writeln!(&mut self.writer, "  slt a0, t0, t1").unwrap();
                    }
                    BinOp::Le => {
                        writeln!(&mut self.writer, "  slt a0, t1, t0").unwrap();
                        writeln!(&mut self.writer, "  xori a0, a0, 1").unwrap();
                    }
                    BinOp::Comma => {
                        writeln!(&mut self.writer, "  mv a0, t1").unwrap();
                    }
                    BinOp::LogOr => {
                        self.count += 1;
                        writeln!(&mut self.writer, "  li a0, 1").unwrap();
                        writeln!(&mut self.writer, "  bne t0, zero, .L.{}", self.count).unwrap();
                        writeln!(&mut self.writer, "  snez a0, t1").unwrap();
                        writeln!(&mut self.writer, ".L.{}:", self.count).unwrap();
                    }
                    BinOp::LogAnd => {
                        self.count += 1;
                        writeln!(&mut self.writer, "  li a0, 0").unwrap();
                        writeln!(&mut self.writer, "  beq t0, zero, .L.{}", self.count).unwrap();
                        writeln!(&mut self.writer, "  snez a0, t1").unwrap();
                        writeln!(&mut self.writer, ".L.{}:", self.count).unwrap();
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
                let count = self.count;

                if let Some(init) = init {
                    self.gen_stmt(*init);
                }
                writeln!(&mut self.writer, ".L.begin.{count}:").unwrap();
                if let Some(cond) = cond {
                    self.gen_expr(*cond);
                    writeln!(&mut self.writer, "  beq a0, zero, .L.end.{count}").unwrap();
                }
                self.gen_stmt(*then);
                if let Some(inc) = inc {
                    self.gen_expr(*inc);
                }
                writeln!(&mut self.writer, "  j .L.begin.{count}").unwrap();
                writeln!(&mut self.writer, ".L.end.{count}:").unwrap();
            }
            TypedNodeKind::If { cond, then, els } => {
                self.count += 1;
                let count = self.count;

                self.gen_expr(*cond);
                writeln!(&mut self.writer, "  beq a0, zero, .L.else.{count}").unwrap();

                self.gen_stmt(*then);
                writeln!(&mut self.writer, "  j .L.end.{count}").unwrap();
                writeln!(&mut self.writer, ".L.else.{count}:").unwrap();

                if let Some(els) = els {
                    self.gen_stmt(*els);
                }
                writeln!(&mut self.writer, ".L.end.{count}:").unwrap();
            }
            TypedNodeKind::Block(nodes) => {
                for node in nodes {
                    self.gen_stmt(node);
                }
            }
            TypedNodeKind::Return(node) => {
                self.gen_expr(*node);
                writeln!(
                    &mut self.writer,
                    "  j .L.return.{}",
                    self.current_fn_name.unwrap()
                )
                .unwrap();
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

pub fn align_to(n: usize, align: usize) -> usize {
    n.div_ceil(align) * align
}

fn push(writer: &mut Box<dyn Write>, reg: &str) {
    writeln!(writer, "  # push {reg}").unwrap();
    writeln!(writer, "  addi sp, sp, -8").unwrap();
    writeln!(writer, "  sd {reg}, 0(sp)").unwrap();
}

fn pop(writer: &mut Box<dyn Write>, reg: &str) {
    writeln!(writer, "  # pop {reg}").unwrap();
    writeln!(writer, "  ld {reg}, 0(sp)").unwrap();
    writeln!(writer, "  addi sp, sp, 8").unwrap();
}

fn load(writer: &mut Box<dyn Write>, ty: &CTypeRef) {
    if let CTypeKind::Array { .. } = ty.borrow().kind {
        return;
    }

    match ty.borrow().size {
        1 => writeln!(writer, "  lb a0, 0(a0)").unwrap(),
        4 => writeln!(writer, "  lw a0, 0(a0)").unwrap(),
        _ => writeln!(writer, "  ld a0, 0(a0)").unwrap(),
    }
}

fn store(writer: &mut Box<dyn Write>, ty: &CTypeRef) {
    pop(writer, "a1");

    match ty.borrow().size {
        1 => writeln!(writer, "  sb a0, 0(a1)").unwrap(),
        4 => writeln!(writer, "  sw a0, 0(a1)").unwrap(),
        _ => writeln!(writer, "  sd a0, 0(a1)").unwrap(),
    }
}

fn addi(writer: &mut Box<dyn Write>, reg1: &str, reg2: &str, imm: i32) {
    if (-2048..=2047).contains(&imm) {
        writeln!(writer, "  addi {reg1}, {reg2}, {imm}").unwrap();
    } else {
        // FIXME: もうちょっと低コストな方法がありそう
        push(writer, "t0");
        writeln!(writer, "  li t0,{imm}").unwrap();
        writeln!(writer, "  add {reg1}, {reg2}, t0").unwrap();
        pop(writer, "t0");
    }
}
