use std::collections::BTreeSet;

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
        log::debug!("[inverse] {self:?}");
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
        if subexprs.len() == 1 {
            *subexprs
                .first()
                .unwrap()
                .clone()
        } else {
            Expr::And(
                subexprs
                    .iter()
                    .map(|subexpr| subexpr.clone().normal())
                    .flat_map(|subexpr| match subexpr {
                        Expr::And(subexprs) => subexprs,
                        expr => BTreeSet::from([expr.into()]),
                    })
                    .collect(),
            )
        }
    }

    fn normal_or(subexprs: BTreeSet<Box<Expr>>) -> Expr {
        if subexprs.len() == 1 {
            *subexprs
                .first()
                .unwrap()
                .clone()
        } else {
            Expr::Or(
                subexprs
                    .iter()
                    .map(|subexpr| subexpr.clone().normal())
                    .flat_map(|subexpr| match subexpr {
                        Expr::Or(subexprs) => subexprs,
                        expr => BTreeSet::from([expr.into()]),
                    })
                    .collect(),
            )
        }
    }

    pub fn normal(&self) -> Self {
        log::debug!("[normal] {self:?}");
        match self {
            Expr::And(subexprs) => Expr::normal_and(subexprs.clone()),
            Expr::Or(subexprs) => Expr::normal_or(subexprs.clone()),
            Expr::Not(expr) => expr.inverse().normal(),
            Expr::Atom(name) => Expr::Atom(name.to_string()),
            Expr::NotAtom(name) => Expr::NotAtom(name.to_string()),
        }
    }

    pub fn names(&self) -> BTreeSet<String> {
        log::debug!("[names] {self:?}");
        match self {
            Expr::And(subexprs) | Expr::Or(subexprs) => {
                subexprs.iter().flat_map(|expr| expr.names()).collect()
            }
            Expr::Atom(name) | Expr::NotAtom(name) => BTreeSet::from([name.to_string()]),
            Expr::Not(expr) => expr.names(),
        }
    }

    pub fn atoms(&self) -> BTreeSet<&Expr> {
        log::debug!("[atoms] {self:?}");
        match self {
            Expr::And(subexprs) | Expr::Or(subexprs) => {
                subexprs.iter().flat_map(|expr| expr.atoms()).collect()
            }
            Expr::Atom(_) | Expr::NotAtom(_) => BTreeSet::from([self]),
            Expr::Not(expr) => expr.atoms(),
        }
    }

    pub fn subexprs(&self) -> BTreeSet<&Expr> {
        log::debug!("[subexprs] {self:?}");
        self.lineaged_subexprs()
            .iter()
            .map(|lineage| lineage[0])
            .collect()
    }

    pub fn lineaged_subexprs(&self) -> BTreeSet<Vec<&Expr>> {
        log::debug!("[lineaged-subexprs] {self:?}");
        match self {
            Expr::And(exprs) | Expr::Or(exprs) => exprs
                .iter()
                .flat_map(|expr| expr.lineaged_subexprs())
                .map(|lineage| [lineage, vec![self]].concat())
                .chain([vec![self]])
                .collect(),
            Expr::Atom(_) | Expr::NotAtom(_) => BTreeSet::from([vec![self]]),
            Expr::Not(_) => panic!("CBA"),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{parser::Parseable, test_init};

    use super::*;

    #[test]
    fn test_axiom() {
        test_init();

        let expr = Expr::parse("a > a").unwrap().normal();
        log::debug!("{:?}", expr.lineaged_subexprs());
    }
}
