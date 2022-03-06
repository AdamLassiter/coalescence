use crate::Set;


// TODO: This could be arena-allocated
// i.e. store vec walk of tree and tree of vec indexes
#[derive(Ord, Eq, PartialOrd, PartialEq, Debug, Clone)]
pub enum Expr {
    And(Set<Box<Expr>>),
    Or(Set<Box<Expr>>),
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
        log::trace!("[inverse] {self:?}");
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

    fn normal_and(subexprs: Set<Box<Expr>>) -> Expr {
        log::trace!("[normal-and] {subexprs:?}");
        let norm: Set<Box<Expr>> = subexprs
            .iter()
            .map(|subexpr| subexpr.clone().normal())
            .flat_map(|subexpr| match subexpr {
                Expr::And(subexprs) => subexprs,
                expr => Set::from([expr.into()]),
            })
            .collect();
        if norm.len() == 1 {
            *norm
                .first()
                .unwrap()
                .clone()
        } else {
            Expr::And(norm)
        }
    }

    fn normal_or(subexprs: Set<Box<Expr>>) -> Expr {
        log::trace!("[normal-or] {subexprs:?}");
        let norm: Set<Box<Expr>> = subexprs
            .iter()
            .map(|subexpr| subexpr.clone().normal())
            .flat_map(|subexpr| match subexpr {
                Expr::Or(subexprs) => subexprs,
                expr => Set::from([expr.into()]),
            })
            .collect();
        if norm.len() == 1 {
            *norm
                .first()
                .unwrap()
                .clone()
        } else {
            Expr::Or(norm)
        }
    }

    pub fn normal(&self) -> Self {
        log::trace!("[normal] {self:?}");
        match self {
            Expr::And(subexprs) => Expr::normal_and(subexprs.clone()),
            Expr::Or(subexprs) => Expr::normal_or(subexprs.clone()),
            Expr::Not(expr) => expr.inverse().normal(),
            Expr::Atom(name) => Expr::Atom(name.to_string()),
            Expr::NotAtom(name) => Expr::NotAtom(name.to_string()),
        }
    }

    pub fn names(&self) -> Set<String> {
        log::trace!("[names] {self:?}");
        match self {
            Expr::And(subexprs) | Expr::Or(subexprs) => {
                subexprs.iter().flat_map(|expr| expr.names()).collect()
            }
            Expr::Atom(name) | Expr::NotAtom(name) => Set::from([name.to_string()]),
            Expr::Not(expr) => expr.names(),
        }
    }

    pub fn atoms(&self) -> Set<&Expr> {
        log::trace!("[atoms] {self:?}");
        match self {
            Expr::And(subexprs) | Expr::Or(subexprs) => {
                subexprs.iter().flat_map(|expr| expr.atoms()).collect()
            }
            Expr::Atom(_) | Expr::NotAtom(_) => Set::from([self]),
            Expr::Not(expr) => expr.atoms(),
        }
    }

    pub fn subexprs(&self) -> Set<&Expr> {
        log::trace!("[subexprs] {self:?}");
        self.lineage()
            .iter()
            .map(|lineage| lineage[0])
            .collect()
    }

    pub fn lineage(&self) -> Set<Vec<&Expr>> {
        log::trace!("[lineaged-subexprs] {self:?}");
        match self {
            Expr::And(exprs) | Expr::Or(exprs) => exprs
                .iter()
                .flat_map(|expr| expr.lineage())
                .map(|lineage| [lineage, vec![self]].concat())
                .chain([vec![self]])
                .collect(),
            Expr::Atom(_) | Expr::NotAtom(_) => Set::from([vec![self]]),
            Expr::Not(_) => panic!("CBA"),
        }
    }
}


#[cfg(test)]
mod tests {
    use crate::{parser::Parseable, log_init};

    use super::*;

    #[test]
    fn normal() -> Result<(), String> {
        log_init();

        // idempotent
        assert_eq!(Expr::parse("a | a")?.normal(),
            Expr::parse("a")?);

        assert_eq!(Expr::parse("a & a")?.normal(),
            Expr::parse("a")?);

        // inverse
        assert_eq!(Expr::parse("~(~a)")?.normal(),
            Expr::parse("a")?);

        // commutative
        assert_eq!(Expr::parse("(a | b) | c")?.normal(),
            Expr::parse("a | (b | c)")?.normal());

        assert_eq!(Expr::parse("(a & b) & c")?.normal(),
            Expr::parse("a & (b & c)")?.normal());

        // idempotent commutative
        assert_eq!(Expr::parse("((a & a) & a) & (a & a) | a")?.normal(),
            Expr::parse("a")?);
        assert_eq!(Expr::parse("((a | a) | a) | (a | a) & a")?.normal(),
            Expr::parse("a")?);

        // syntactic sugar
        assert_eq!(Expr::parse("a > b")?.normal(),
            Expr::parse("~a | b")?.normal());

        assert_eq!(Expr::parse("a = b")?.normal(),
            Expr::parse("(a > b) & (b > a)")?.normal());
        assert_eq!(Expr::parse("a = b")?.normal(),
            Expr::parse("(~a | b) & (~b | a)")?.normal());

        Ok(())
    }

    #[test]
    fn inverse() -> Result<(), String> {
        log_init();

        Ok(())
    }

    #[test]
    fn names() -> Result<(), String> {
        log_init();

        Ok(())
    }

    #[test]
    fn atoms() -> Result<(), String> {
        log_init();

        Ok(())
    }

    #[test]
    fn subexprs() -> Result<(), String> {
        log_init();

        Ok(())
    }

    #[test]
    fn lineage() -> Result<(), String> {
        log_init();

        Ok(())
    }
}
