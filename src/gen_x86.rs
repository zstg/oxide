use crate::gen_ir::{Function, IROp, IR};
use crate::util::roundup;
use crate::{Scope, Var, REGS_N};

const REGS: [&str; REGS_N] = ["r10", "r11", "rbx", "r12", "r13", "r14", "r15"];
const REGS8: [&str; REGS_N] = ["r10b", "r11b", "bl", "r12b", "r13b", "r14b", "r15b"];
const REGS32: [&str; REGS_N] = ["r10d", "r11d", "ebx", "r12d", "r13d", "r14d", "r15d"];

// AVX512 registers
const ZMM_REGS: [&str; 32] = [
    "zmm0", "zmm1", "zmm2", "zmm3", "zmm4", "zmm5", "zmm6", "zmm7",
    "zmm8", "zmm9", "zmm10", "zmm11", "zmm12", "zmm13", "zmm14", "zmm15",
    "zmm16", "zmm17", "zmm18", "zmm19", "zmm20", "zmm21", "zmm22", "zmm23",
    "zmm24", "zmm25", "zmm26", "zmm27", "zmm28", "zmm29", "zmm30", "zmm31"
];

use std::sync::Mutex;

// Quoted from oxide
// > This pass generates x86-64 assembly from IR.

const ARGREGS: [&str; 6] = ["rdi", "rsi", "rdx", "rcx", "r8", "r9"];
const ARGREGS8: [&str; 6] = ["dil", "sil", "dl", "cl", "r8b", "r9b"];
const ARGREGS32: [&str; 6] = ["edi", "esi", "edx", "ecx", "r8d", "r9d"];

lazy_static! {
    static ref LABEL: Mutex<usize> = Mutex::new(0);
}

fn backslash_escape(s: String, len: usize) -> String {
    let mut sb = String::new();
    for i in 0..len {
        if let Some(c) = s.chars().collect::<Vec<char>>().get(i) {
            // Issue: https://github.com/rust-lang/rfcs/issues/751
            let escaped = match c {
                // '\b' => Some('b'),
                // '\f' => Some('f'),
                '\n' => Some('n'),
                '\r' => Some('r'),
                '\t' => Some('t'),
                '\\' => Some('\\'),
                '\'' => Some('\''),
                '\"' => Some('\"'),
                _ => None,
            };
            if let Some(esc) = escaped {
                sb.push('\\');
                sb.push(esc);
            } else if c.is_ascii_graphic() || c == &' ' {
                sb.push(*c);
            } else {
                sb.push_str(&format!("\\{:o}", *c as i8));
            }
            if i == len - 1 {
                sb.push_str("\\000");
            }
        } else {
            sb.push_str("\\000");
        }
    }
    sb
}

macro_rules! emit{
    ($fmt:expr) => (print!(concat!("\t", $fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => (print!(concat!("\t", $fmt, "\n"), $($arg)*));
}

fn emit_cmp(ir: IR, insn: &'static str) {
    let lhs = ir.lhs.unwrap();
    let rhs = ir.rhs.unwrap();
    emit!("cmp {}, {}", REGS[lhs], REGS[rhs]);
    emit!("{} {}", insn, REGS8[lhs]);
    emit!("movzb {}, {}", REGS[lhs], REGS8[lhs]);
}

fn reg(r: usize, size: u8) -> &'static str {
    match size {
        1 => REGS8[r],
        4 => REGS32[r],
        8 => REGS[r],
        _ => unreachable!(),
    }
}

fn argreg(r: usize, size: u8) -> &'static str {
    match size {
        1 => ARGREGS8[r],
        4 => ARGREGS32[r],
        8 => ARGREGS[r],
        _ => unreachable!(),
    }
}

fn emit_header() {
    println!("bits 64");
    println!("section .text");
    println!("global main");
    println!("extern printf");
    println!("extern exit");
    println!();
}

fn gen(f: Function) {
    use self::IROp::*;
    let ret = format!(".Lend{}", *LABEL.lock().unwrap());
    *LABEL.lock().unwrap() += 1;

    println!(".text");
    println!(".global {}", f.name);
    println!("{}:", f.name);
    emit!("push rbp");
    emit!("mov rbp, rsp");
    emit!("sub rsp, {}", roundup(f.stacksize, 64));  // Align to 64 bytes for AVX512
    emit!("push r12");
    emit!("push r13");
    emit!("push r14");
    emit!("push r15");

    for ir in f.ir {
        let lhs = ir.lhs.unwrap_or(0);
        let rhs = ir.rhs.unwrap_or(0);
        match ir.op {
            Imm => emit!("mov {}, {}", REGS[lhs], rhs as i32),
            Mov => emit!("mov {}, {}", REGS[lhs], REGS[rhs]),
            Return => {
                emit!("mov rax, {}", REGS[lhs]);
                emit!("jmp {}", ret);
            }
            Call(name, nargs, args) => {
                for i in 0..nargs {
                    emit!("mov {}, {}", ARGREGS[i], REGS[args[i]]);
                }
                emit!("push r10");
                emit!("push r11");
                emit!("mov rax, 0");
                emit!("call {}", name);
                emit!("pop r11");
                emit!("pop r10");

                emit!("mov {}, rax", REGS[lhs]);
            }
            Label => println!(".L{}:", lhs),
            LabelAddr(name) => emit!("lea {}, {}", REGS[lhs], name),
            Neg => emit!("neg {}", REGS[lhs]),
            EQ => emit_cmp(ir, "sete"),
            NE => emit_cmp(ir, "setne"),
            LT => emit_cmp(ir, "setl"),
            LE => emit_cmp(ir, "setle"),
            AND => emit!("and {}, {}", REGS[lhs], REGS[rhs]),
            OR => emit!("or {}, {}", REGS[lhs], REGS[rhs]),
            XOR => emit!("xor {}, {}", REGS[lhs], REGS[rhs]),
            SHL => {
                emit!("mov cl, {}", REGS8[rhs]);
                emit!("shl {}, cl", REGS[lhs]);
            }
            SHR => {
                emit!("mov cl, {}", REGS8[rhs]);
                emit!("shr {}, cl", REGS[lhs]);
            }
            Mod => {
                /* Same meaning(?).
                 * emit!("mov rdx, 0");
                 * emit!("mov rax, {}", REGS[lhs]);
                 */
                emit!("mov rax, {}", REGS[lhs]);
                emit!("cqo"); // rax -> rdx:rax
                emit!("idiv {}", REGS[rhs]);
                emit!("mov {}, rdx", REGS[lhs]);
            }
            Jmp => emit!("jmp .L{}", lhs),
            If => {
                emit!("cmp {}, 0", REGS[lhs]);
                emit!("jne .L{}", rhs);
            }
            Unless => {
                emit!("cmp {}, 0", REGS[lhs]);
                emit!("je .L{}", rhs);
            }
            Load(size) => {
                match size {
                    1 => emit!("movzx {}, byte [{}]", REGS[lhs], REGS[rhs]),
                    4 => emit!("movsxd {}, dword [{}]", REGS[lhs], REGS[rhs]),
                    8 => emit!("mov {}, [{}]", REGS[lhs], REGS[rhs]),
                    _ => panic!("Unknown data size: {}", size),
                }
            }
            Store(size) => {
                match size {
                    1 => emit!("mov byte [{}], {}", REGS[lhs], REGS8[rhs]),
                    4 => emit!("mov dword [{}], {}", REGS[lhs], REGS32[rhs]),
                    8 => emit!("mov [{}], {}", REGS[lhs], REGS[rhs]),
                    _ => panic!("Unknown data size: {}", size),
                }
            }
            StoreArg(size) => {
                match size {
                    1 => emit!("mov byte [rbp+{}], {}", lhs, REGS8[rhs]),
                    4 => emit!("mov dword [rbp+{}], {}", lhs, REGS32[rhs]),
                    8 => emit!("mov qword [rbp+{}], {}", lhs, REGS[rhs]),
                    _ => panic!("Unknown data size: {}", size),
                }
            }
            Add => emit!("add {}, {}", REGS[lhs], REGS[rhs]),
            AddImm => emit!("add {}, {}", REGS[lhs], rhs),
            Sub => emit!("sub {}, {}", REGS[lhs], REGS[rhs]),
            SubImm => emit!("sub {}, {}", REGS[lhs], rhs),
            Bprel => emit!("lea {}, [rbp+{}]", REGS[lhs], rhs),
            Mul => {
                emit!("mov rax, {}", REGS[rhs]);
                emit!("mul {}", REGS[lhs]);
                emit!("mov {}, rax", REGS[lhs]);
            }
            MulImm => emit!("imul {}, {}, {}", REGS[lhs], REGS[lhs], rhs),
            Div => {
                emit!("mov rax, {}", REGS[lhs]);
                emit!("cqo");
                emit!("idiv {}", REGS[rhs]);
                emit!("mov {}, rax", REGS[lhs]);
            }
            Nop | Kill => (),
            AVX512Add => emit!("vaddpd {}, {}, {}", ZMM_REGS[lhs], ZMM_REGS[lhs], ZMM_REGS[rhs]),
            AVX512Sub => emit!("vsubpd {}, {}, {}", ZMM_REGS[lhs], ZMM_REGS[lhs], ZMM_REGS[rhs]),
            AVX512Mul => emit!("vmulpd {}, {}, {}", ZMM_REGS[lhs], ZMM_REGS[lhs], ZMM_REGS[rhs]),
            AVX512Div => emit!("vdivpd {}, {}, {}", ZMM_REGS[lhs], ZMM_REGS[lhs], ZMM_REGS[rhs]),
            AVX512Load => {
                // Determine if we're loading from a memory address or register
                if ir.rhs.is_some() {
                    emit!("vmovapd {}, [{}]", ZMM_REGS[lhs], REGS[rhs]);
                } else {
                    emit!("vmovapd {}, [rsp+{}]", ZMM_REGS[lhs], lhs * 8);
                }
            },
            AVX512Store => {
                // Determine if we're storing to a memory address or register
                if ir.lhs.is_some() {
                    emit!("vmovapd [{}], {}", REGS[lhs], ZMM_REGS[rhs]);
                } else {
                    emit!("vmovapd [rsp+{}], {}", lhs * 8, ZMM_REGS[rhs]);
                }
            },
            AVX512Mov => emit!("vmovapd {}, {}", ZMM_REGS[lhs], ZMM_REGS[rhs]),
            AVX512Addi => emit!("vpaddd {}, {}, {}", ZMM_REGS[lhs], ZMM_REGS[lhs], ZMM_REGS[rhs]),
            AVX512Subi => emit!("vpsubd {}, {}, {}", ZMM_REGS[lhs], ZMM_REGS[lhs], ZMM_REGS[rhs]),
            AVX512Muli => emit!("vpmulld {}, {}, {}", ZMM_REGS[lhs], ZMM_REGS[lhs], ZMM_REGS[rhs]),
            AVX512Loadi => emit!("vmovdqu32 {}, [{}]", ZMM_REGS[lhs], REGS[rhs]),
            AVX512Storei => emit!("vmovdqu32 [{}], {}", REGS[lhs], ZMM_REGS[rhs]),
            AVX512Movi => emit!("vmovdqu32 {}, {}", ZMM_REGS[lhs], ZMM_REGS[rhs]),
            AVX512Zero => emit!("vpxord {}, {}, {}", ZMM_REGS[lhs], ZMM_REGS[lhs], ZMM_REGS[lhs]),
            AVX512Set1 => emit!("vbroadcastsd {}, {}", ZMM_REGS[lhs], REGS[rhs]),
            AVX512Set1i => emit!("vpbroadcastd {}, {}", ZMM_REGS[lhs], REGS[rhs]),
            AVX512Cmplt => emit!("vcmpltpd k1, {}, {}", ZMM_REGS[lhs], ZMM_REGS[rhs]),
            AVX512Cmple => emit!("vcmplepd k1, {}, {}", ZMM_REGS[lhs], ZMM_REGS[rhs]),
            AVX512Cmpeq => emit!("vcmpeqpd k1, {}, {}", ZMM_REGS[lhs], ZMM_REGS[rhs]),
            AVX512MaskMove => emit!("vmovapd {} {{k1}}, {}", ZMM_REGS[lhs], ZMM_REGS[rhs]),
            AVX512MaskLoad => emit!("vmovapd {} {{k1}}, [{}]", ZMM_REGS[lhs], REGS[rhs]),
            AVX512MaskStore => emit!("vmovapd [{}] {{k1}}, {}", REGS[lhs], ZMM_REGS[rhs]),
            AVX512Cvtdq2pd => emit!("vcvtdq2pd {}, {}", ZMM_REGS[lhs], ZMM_REGS[rhs].replace("zmm", "ymm")),
            AVX512Cvtpd2dq => emit!("vcvtpd2dq {}, {}", ZMM_REGS[lhs].replace("zmm", "ymm"), ZMM_REGS[rhs]),
            AVX512Extract => emit!("vmovq {}, {}", REGS[lhs], ZMM_REGS[rhs].replace("zmm", "xmm")),
            AVX512Insert => emit!("vpinsrq {}, {}, {}, 0", ZMM_REGS[lhs].replace("zmm", "xmm"), ZMM_REGS[lhs].replace("zmm", "xmm"), REGS[rhs]),
        }
    }

    println!("{}:", ret);
    emit!("pop r15");
    emit!("pop r14");
    emit!("pop r13");
    emit!("pop r12");
    emit!("mov rsp, rbp");
    emit!("pop rbp");
    emit!("ret");
}

pub fn gen_x86(globals: Vec<Var>, fns: Vec<Function>) {
    // Extract global variables for data section
    let mut globals_data = Vec::new();
    for var in &globals {
        if let Scope::Global(ref data, len, is_extern) = var.scope {
            if !is_extern {
                globals_data.push((var.name.clone(), data.clone(), len));
            }
        }
    }
    
    emit_header();
    
    // Emit data section if we have globals
    if !globals_data.is_empty() {
        println!("section .data");
        for (name, data, len) in globals_data {
            println!("{}:", name);
            if data.is_empty() {
                println!("    dq 0");
            } else {
                // Handle string literals or other initialized data
                println!("    db {}", data);
            }
        }
        println!();
    }
    
    // Emit text section
    println!("section .text");
    
    for f in fns {
        println!("{}:", f.name);
        println!("    push rbp");
        println!("    mov rbp, rsp");
        println!("    sub rsp, {}", f.stacksize);
        
        gen(f);
        
        // Add a default return if needed
        println!("    leave");
        println!("    ret");
        println!();
    }
}
