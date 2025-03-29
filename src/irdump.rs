use crate::gen_ir::{Function, IROp, IRType, IR};

use std::fmt;

#[derive(Clone, Debug)]
pub struct IRInfo {
    name: &'static str,
    pub ty: IRType,
}

impl IRInfo {
    pub fn new(name: &'static str, ty: IRType) -> Self {
        IRInfo { name, ty }
    }

    pub fn from(op: &IROp) -> IRInfo {
        use self::IROp::*;
        match *op {
            Add => IRInfo::new("ADD", IRType::RegReg),
            AddImm => IRInfo::new("ADD", IRType::RegImm),
            Call(_, _, _) => IRInfo::new("CALL", IRType::Call),
            Div => IRInfo::new("DIV", IRType::RegReg),
            Imm => IRInfo::new("MOV", IRType::RegImm),
            Jmp => IRInfo::new("JMP", IRType::Jmp),
            Kill => IRInfo::new("KILL", IRType::Reg),
            Label => IRInfo::new("", IRType::Label),
            LabelAddr(_) => IRInfo::new("LABEL_ADDR", IRType::LabelAddr),
            EQ => IRInfo::new("EQ", IRType::RegReg),
            NE => IRInfo::new("NE", IRType::RegReg),
            LE => IRInfo::new("LE", IRType::RegReg),
            LT => IRInfo::new("LT", IRType::RegReg),
            AND => IRInfo::new("AND", IRType::RegReg),
            OR => IRInfo::new("OR", IRType::RegReg),
            XOR => IRInfo::new("XOR", IRType::RegReg),
            SHL => IRInfo::new("SHL", IRType::RegReg),
            SHR => IRInfo::new("SHR", IRType::RegReg),
            Mod => IRInfo::new("MOD", IRType::RegReg),
            Neg => IRInfo::new("NEG", IRType::Reg),
            Load(_) => IRInfo::new("LOAD", IRType::Mem),
            Mov => IRInfo::new("MOV", IRType::RegReg),
            Mul => IRInfo::new("MUL", IRType::RegReg),
            MulImm => IRInfo::new("MUL", IRType::RegImm),
            Nop => IRInfo::new("NOP", IRType::Noarg),
            Return => IRInfo::new("RET", IRType::Reg),
            Store(_) => IRInfo::new("STORE", IRType::Mem),
            StoreArg(_) => IRInfo::new("STORE_ARG", IRType::StoreArg),
            Sub => IRInfo::new("SUB", IRType::RegReg),
            SubImm => IRInfo::new("SUB", IRType::RegImm),
            Bprel => IRInfo::new("BPREL", IRType::RegImm),
            If => IRInfo::new("IF", IRType::RegLabel),
            Unless => IRInfo::new("UNLESS", IRType::RegLabel),
            AVX512Add => IRInfo::new("AVX512_ADD", IRType::RegReg),
            AVX512Sub => IRInfo::new("AVX512_SUB", IRType::RegReg),
            AVX512Mul => IRInfo::new("AVX512_MUL", IRType::RegReg),
            AVX512Div => IRInfo::new("AVX512_DIV", IRType::RegReg),
            AVX512Load => IRInfo::new("AVX512_LOAD", IRType::Mem),
            AVX512Store => IRInfo::new("AVX512_STORE", IRType::Mem),
            AVX512Mov => IRInfo::new("AVX512_MOV", IRType::RegReg),
            AVX512Addi => IRInfo::new("AVX512_ADDI", IRType::RegReg),
            AVX512Subi => IRInfo::new("AVX512_SUBI", IRType::RegReg),
            AVX512Muli => IRInfo::new("AVX512_MULI", IRType::RegReg),
            AVX512Loadi => IRInfo::new("AVX512_LOADI", IRType::Mem),
            AVX512Storei => IRInfo::new("AVX512_STOREI", IRType::Mem),
            AVX512Movi => IRInfo::new("AVX512_MOVI", IRType::RegReg),
            AVX512Zero => IRInfo::new("AVX512_ZERO", IRType::Reg),
            AVX512Set1 => IRInfo::new("AVX512_SET1", IRType::RegReg),
            AVX512Set1i => IRInfo::new("AVX512_SET1I", IRType::RegReg),
            AVX512Cmplt => IRInfo::new("AVX512_CMPLT", IRType::RegReg),
            AVX512Cmple => IRInfo::new("AVX512_CMPLE", IRType::RegReg),
            AVX512Cmpeq => IRInfo::new("AVX512_CMPEQ", IRType::RegReg),
            AVX512MaskMove => IRInfo::new("AVX512_MASK_MOV", IRType::RegReg),
            AVX512MaskLoad => IRInfo::new("AVX512_MASK_LOAD", IRType::Mem),
            AVX512MaskStore => IRInfo::new("AVX512_MASK_STORE", IRType::Mem),
            AVX512Cvtdq2pd => IRInfo::new("AVX512_CVTDQ2PD", IRType::RegReg),
            AVX512Cvtpd2dq => IRInfo::new("AVX512_CVTPD2DQ", IRType::RegReg),
            AVX512Extract => IRInfo::new("AVX512_EXTRACT", IRType::RegReg),
            AVX512Insert => IRInfo::new("AVX512_INSERT", IRType::RegReg),
        }
    }
}

impl<'a> From<&'a IROp> for IRInfo {
    fn from(op: &'a IROp) -> IRInfo {
        IRInfo::from(op)
    }
}

impl fmt::Display for IR {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::IRType::*;

        let info = &IRInfo::from(&self.op);

        let lhs = self.lhs.unwrap();
        match info.ty {
            Label => write!(f, ".L{}:", lhs),
            LabelAddr => match self.op {
                IROp::LabelAddr(ref name) => write!(f, "  {} r{}, {}", info.name, lhs, name),
                _ => unreachable!(),
            },
            Imm => write!(f, "  {} {}", info.name, lhs),
            Reg => write!(f, "  {} r{}", info.name, lhs),
            Jmp => write!(f, "  {} .L{}", info.name, lhs),
            RegReg => write!(f, "  {} r{}, r{}", info.name, lhs, self.rhs.unwrap()),
            Mem | StoreArg => match self.op {
                IROp::Load(ref size) | IROp::Store(ref size) => {
                    write!(f, "  {}{} r{}, {}", info.name, size, lhs, self.rhs.unwrap())
                }
                IROp::StoreArg(ref size) => {
                    write!(f, "  {}{} {}, {}", info.name, size, lhs, self.rhs.unwrap())
                }
                _ => unreachable!(),
            },
            RegImm => write!(f, "  {} r{}, {}", info.name, lhs, self.rhs.unwrap() as i32),
            RegLabel => write!(f, "  {} r{}, .L{}", info.name, lhs, self.rhs.unwrap()),
            Call => match self.op {
                IROp::Call(ref name, nargs, args) => {
                    let mut sb: String = format!("  r{} = {}(", lhs, name);
                    for (i, arg) in args.iter().enumerate().take(nargs) {
                        if i != 0 {
                            sb.push_str(", ");
                        }
                        sb.push_str(&format!("r{}", *arg));
                    }
                    sb.push(')');
                    write!(f, "{}", sb)
                }
                _ => unreachable!(),
            },
            Noarg => write!(f, "  {}", info.name),
        }
    }
}

pub fn dump_ir(fns: &[Function]) {
    for f in fns {
        eprintln!("{}(): ", f.name);
        for ir in &f.ir {
            eprintln!("{}", ir);
        }
    }
}
