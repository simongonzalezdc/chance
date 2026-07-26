/// Abstract syntax tree for dice notation.
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Sum(Vec<(Sign, Term)>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sign {
    Plus,
    Minus,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Term {
    Number(i64),
    Dice(DiceTerm),
}

#[derive(Debug, Clone, PartialEq)]
pub struct DiceTerm {
    pub count: u64,
    pub size: DieSize,
    pub modifiers: Vec<Modifier>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DieSize {
    Sides(u64),
    Percentile,
    Fudge,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Modifier {
    KeepHighest(u64),
    KeepLowest(u64),
    DropHighest(u64),
    DropLowest(u64),
    Explode,
    ExplodeCompound,
    Reroll {
        comparator: Comparator,
        value: i64,
        once: bool,
    },
    Success {
        comparator: Comparator,
        value: i64,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Comparator {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

impl Comparator {
    pub fn compare(&self, a: i64, b: i64) -> bool {
        match self {
            Comparator::Eq => a == b,
            Comparator::Ne => a != b,
            Comparator::Lt => a < b,
            Comparator::Le => a <= b,
            Comparator::Gt => a > b,
            Comparator::Ge => a >= b,
        }
    }
}
