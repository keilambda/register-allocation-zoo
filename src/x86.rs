use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Name(pub String);

#[derive(Debug, Clone)]
pub struct Label(pub String);

#[derive(Debug, Clone)]
pub struct Arity(pub i8);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Register {
    RSP,
    RBP,
    RAX,
    RBX,
    RCX,
    RDX,
    RSI,
    RDI,
    R8,
    R9,
    R10,
    R11,
    R12,
    R13,
    R14,
    R15,
}

impl Register {
    pub const CALLER_SAVED: &[Register] = &[
        Register::RAX,
        Register::RCX,
        Register::RDX,
        Register::RSI,
        Register::RDI,
        Register::R8,
        Register::R9,
        Register::R10,
        Register::R11,
    ];

    pub const CALLEE_SAVED: &[Register] = &[
        Register::RSP,
        Register::RBP,
        Register::RBX,
        Register::R12,
        Register::R13,
        Register::R14,
        Register::R15,
    ];

    pub const ARGUMENT_PASSING: &[Register] = &[
        Register::RDI,
        Register::RSI,
        Register::RDX,
        Register::RCX,
        Register::R8,
        Register::R9,
    ];
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Operand {
    Reg(Register),
    Mem(Register, i64),
    Var(Name),
    Imm(i64),
}

#[derive(Debug, Clone)]
pub enum InstrF<A> {
    AddQ(A, A),
    SubQ(A, A),
    NegQ(A),
    MovQ(A, A),
    PushQ(A),
    PopQ(A),
    CallQ(Label, Arity),
    Jmp(Label),
    Syscall,
    RetQ,
}

pub type Instr = InstrF<Operand>;

pub struct Liveness {
    pub instr: Instr,
    pub live_in: HashSet<Operand>,
    pub live_out: HashSet<Operand>,
}

impl Instr {
    pub fn uses(&self) -> HashSet<Operand> {
        use InstrF::*;
        match self {
            AddQ(src, dst) | SubQ(src, dst) => HashSet::from([src.clone(), dst.clone()]),
            NegQ(dst) => HashSet::from([dst.clone()]),
            MovQ(src, _) => HashSet::from([src.clone()]),
            CallQ(_, arity) => Register::ARGUMENT_PASSING
                .iter()
                .take(arity.0 as usize)
                .map(|reg| Operand::Reg(reg.clone()))
                .collect(),
            PushQ(_) | PopQ(_) | Jmp(_) | Syscall | RetQ => HashSet::default(),
        }
    }

    pub fn defs(&self) -> HashSet<Operand> {
        use InstrF::*;
        match self {
            AddQ(_, dst) | SubQ(_, dst) | NegQ(dst) | MovQ(_, dst) => HashSet::from([dst.clone()]),
            CallQ(_, _) => Register::CALLER_SAVED
                .iter()
                .map(|reg| Operand::Reg(reg.clone()))
                .collect(),
            PushQ(_) | PopQ(_) | Jmp(_) | Syscall | RetQ => HashSet::default(),
        }
    }
}

pub struct Block(pub Vec<Instr>);

impl Block {
    pub fn liveness(&self) -> Vec<Liveness> {
        let mut live_out = HashSet::new();
        let mut liveness = Vec::with_capacity(self.0.len());

        for instr in self.0.iter().rev() {
            let use_set = instr.uses();
            let def_set = instr.defs();

            let mut live_in = use_set.clone();
            live_in.extend(
                live_out
                    .difference(&def_set)
                    .cloned()
                    .collect::<HashSet<_>>(),
            );

            liveness.push(Liveness {
                instr: instr.clone(),
                live_in: live_in.clone(),
                live_out: live_out.clone(),
            });

            live_out = live_in;
        }

        liveness.reverse();
        liveness
    }
}
