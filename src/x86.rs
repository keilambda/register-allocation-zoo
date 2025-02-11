use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Name(pub String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Label(pub String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Arity(pub u8);

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

impl Operand {
    pub fn can_live(&self) -> bool {
        use Operand::*;
        match self {
            Reg(_) | Mem(_, _) | Var(_) => true,
            Imm(_) => false,
        }
    }
}

impl From<&str> for Operand {
    fn from(value: &str) -> Self {
        Operand::Var(Name(value.to_string()))
    }
}

impl From<i64> for Operand {
    fn from(value: i64) -> Self {
        Operand::Imm(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
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

impl Instr {
    pub fn uses(&self) -> HashSet<Operand> {
        use InstrF::*;
        let mut set = HashSet::new();
        match self {
            AddQ(src, dst) | SubQ(src, dst) => {
                if src.can_live() {
                    set.insert(src.clone());
                }
                if dst.can_live() {
                    set.insert(dst.clone());
                }
            }
            NegQ(dst) if dst.can_live() => {
                set.insert(dst.clone());
            }
            MovQ(src, _) if src.can_live() => {
                set.insert(src.clone());
            }
            CallQ(_, arity) => {
                set.extend(
                    Register::ARGUMENT_PASSING
                        .iter()
                        .take(arity.0 as usize)
                        .map(|reg| Operand::Reg(reg.clone())),
                );
            }
            _ => {}
        }
        set
    }

    pub fn defs(&self) -> HashSet<Operand> {
        use InstrF::*;
        let mut set = HashSet::new();
        match self {
            AddQ(_, dst) | SubQ(_, dst) | NegQ(dst) | MovQ(_, dst) if dst.can_live() => {
                set.insert(dst.clone());
            }
            CallQ(_, _) => {
                set.extend(
                    Register::CALLER_SAVED
                        .iter()
                        .map(|reg| Operand::Reg(reg.clone())),
                );
            }
            _ => {}
        }
        set
    }
}

pub struct Liveness {
    pub instr: Instr,
    pub before: HashSet<Operand>,
    pub after: HashSet<Operand>,
}

pub struct Block(pub Vec<Instr>);

impl Block {
    pub fn liveness(&self) -> Vec<Liveness> {
        let mut after = HashSet::new();
        let mut liveness = Vec::with_capacity(self.0.len());

        for instr in self.0.iter().rev() {
            let before: HashSet<_> = instr
                .uses()
                .union(&after.difference(&instr.defs()).cloned().collect())
                .cloned()
                .collect();

            liveness.push(Liveness {
                instr: instr.clone(),
                before: before.clone(),
                after: after.clone(),
            });

            after = before;
        }

        liveness.reverse();
        liveness
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_liveness() {
        use InstrF::*;

        let block = Block(vec![
            MovQ(32.into(), "x".into()),
            MovQ(10.into(), "y".into()),
            MovQ("x".into(), "z".into()),
            AddQ("y".into(), "z".into()),
            NegQ("z".into()),
        ]);

        let analysis = block.liveness();

        let live = analysis.get(0).unwrap();
        assert_eq!(live.instr, *block.0.get(0).unwrap());
        assert_eq!(live.before, HashSet::default());
        assert_eq!(live.after, HashSet::from(["x".into()]));

        let live = analysis.get(1).unwrap();
        assert_eq!(live.instr, *block.0.get(1).unwrap());
        assert_eq!(live.before, HashSet::from(["x".into()]));
        assert_eq!(live.after, HashSet::from(["x".into(), "y".into()]));

        let live = analysis.get(2).unwrap();
        assert_eq!(live.instr, *block.0.get(2).unwrap());
        assert_eq!(live.before, HashSet::from(["x".into(), "y".into()]));
        assert_eq!(live.after, HashSet::from(["y".into(), "z".into()]));

        let live = analysis.get(3).unwrap();
        assert_eq!(live.instr, *block.0.get(3).unwrap());
        assert_eq!(live.before, HashSet::from(["y".into(), "z".into()]));
        assert_eq!(live.after, HashSet::from(["z".into()]));

        let live = analysis.get(4).unwrap();
        assert_eq!(live.instr, *block.0.get(4).unwrap());
        assert_eq!(live.before, HashSet::from(["z".into()]));
        assert_eq!(live.after, HashSet::default());
    }
}
