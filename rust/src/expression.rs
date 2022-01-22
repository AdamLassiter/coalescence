use std::{collections::BTreeSet, fmt::Display};

// TODO: This could be arena-allocated
// i.e. store vec walk of tree and tree of vec indexes
#[derive(Ord, Eq, PartialOrd, PartialEq, Debug, Clone)]
pub enum Expr {
    And(BTreeSet<Box<Expr>>),
    Or(BTreeSet<Box<Expr>>),
    Not(Box<Expr>),
    Atom(String),
    NotAtom(String),
}

impl Expr {
    pub fn and(exprs: &[Expr]) -> Self {
        Self::And(exprs.iter().map(|expr| expr.to_owned().into()).collect())
    }

    pub fn or(exprs: &[Expr]) -> Self {
        Self::Or(exprs.iter().map(|expr| expr.to_owned().into()).collect())
    }

    pub fn not(expr: Expr) -> Self {
        Self::Not(expr.into())
    }

    pub fn inverse(&self) -> Self {
        match self {
            Expr::And(subexprs) => Expr::Or(
                subexprs
                    .iter()
                    .map(|subexpr| subexpr.clone().inverse().into())
                    .collect(),
            ),
            Expr::Or(subexprs) => Expr::And(
                subexprs
                    .iter()
                    .map(|subexpr| subexpr.clone().inverse().into())
                    .collect(),
            ),
            Expr::Not(subexpr) => *subexpr.clone(),
            Expr::Atom(name) => Expr::NotAtom(name.to_string()),
            Expr::NotAtom(name) => Expr::Atom(name.to_string()),
        }
    }

    fn normal_and(subexprs: BTreeSet<Box<Expr>>) -> Expr {
        Expr::And(
            subexprs
                .iter()
                .map(|subexpr| subexpr.clone().normal())
                .flat_map(|subexpr| match subexpr {
                    Expr::And(subexprs) => subexprs,
                    expr => BTreeSet::from([expr.into()]),
                })
                .collect()
            )
    }

    fn normal_or(subexprs: BTreeSet<Box<Expr>>) -> Expr {
        Expr::Or(
            subexprs
                .iter()
                .map(|subexpr| subexpr.clone().normal())
                .flat_map(|subexpr| match subexpr {
                    Expr::Or(subexprs) => subexprs,
                    expr => BTreeSet::from([expr.into()]),
                })
                .collect()
            )
    }

    pub fn normal(&self) -> Self {
        match self {
            Expr::And(subexprs) => Expr::normal_and(subexprs.clone()),
            Expr::Or(subexprs) => Expr::normal_or(subexprs.clone()),
            Expr::Not(expr) => expr.inverse().normal(),
            Expr::Atom(name) => Expr::Atom(name.to_string()),
            Expr::NotAtom(name) => Expr::NotAtom(name.to_string()),
        }
    }

    pub fn names(&self) -> BTreeSet<String> {
        match self {
            Expr::And(subexprs) | Expr::Or(subexprs) => subexprs.iter()
                .flat_map(|expr| expr.names())
                .collect(),
            Expr::Atom(name) | Expr::NotAtom(name) => BTreeSet::from([name.to_string()]),
            Expr::Not(expr) => expr.names(),
        }
    }

    pub fn atoms(&self) -> BTreeSet<&Expr> {
        match self {
            Expr::And(subexprs) | Expr::Or(subexprs) => subexprs.iter()
                .flat_map(|expr| expr.atoms())
                .collect(),
            Expr::Atom(_) | Expr::NotAtom(_) => BTreeSet::from([self]),
            Expr::Not(expr) => expr.atoms(),
        }
    }

    pub fn subexprs(&self) -> BTreeSet<&Expr> {
        self.lineaged_subexprs().iter()
            .map(|lineage| lineage[0])
            .collect()
    }

    pub fn lineaged_subexprs(&self) -> BTreeSet<Vec<&Expr>> {
        match self {
            Expr::And(exprs) | Expr::Or(exprs) => {
                exprs.iter()
                    .flat_map(|expr| expr.lineaged_subexprs())
                    .map(|lineage| [lineage, vec![self]].concat())
                    .chain([vec![self]])
                    .collect()
            },
            Expr::Atom(_) | Expr::NotAtom(_) => BTreeSet::from([vec![self]]),
            Expr::Not(_) => panic!("CBA")
        }
    }
}

impl Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}