pub struct Name(pub String);

pub struct Label(pub String);

pub struct Arity(pub i8);

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

pub enum Operand {
    Reg(Register),
    Mem(Register, i64),
    Var(Name),
    Imm(i64),
}

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
