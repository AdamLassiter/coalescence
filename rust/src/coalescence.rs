use crate::{expression::Expr, SSet, Set};

pub trait Coalesceable: Sized + Ord + Clone + std::fmt::Debug {
    fn axiom_set(&self) -> Set<Self>;

    fn is_axiom(&self) -> bool;

    fn children(&self) -> Set<Box<Self>>;

    fn dim_bound(&self) -> usize;

    fn spawn(&self) -> SSet<Self>;

    fn fire(&self, tokens: SSet<Self>) -> SSet<Self>;

    fn project(&self, tokens: SSet<Self>) -> SSet<Self>;

    fn coalesce(&self) -> Option<SSet<Self>> {
        log::debug!("[coalesce] {self:?}");
        let mut tokens = Self::spawn(self);
        if tokens.is_empty() {
            return None;
        }

        let mut old_tokens = Set::new();
        while !tokens.contains(&Set::from([self.clone()])) {
            log::debug!("[coalesce] {self:?} not in {tokens:?}");
            if old_tokens == tokens {
                let current_dim = tokens.iter().map(Set::len).fold(0, |a, b| a.max(b));
                if current_dim < Self::dim_bound(self) {
                    tokens = Self::project(self, tokens);
                } else {
                    return None;
                }
            }
            old_tokens = tokens.clone();
            tokens = Self::fire(self, tokens);
        }

        Some(tokens)
    }
}

impl Coalesceable for Expr {
    fn axiom_set(&self) -> Set<Self> {
        Set::from([self.clone(), self.inverse().normal()])
    }

    fn is_axiom(&self) -> bool {
        match self {
            Expr::Or(subexprs) => {
                subexprs.first()
                    .map(|subexpr| subexprs.contains(&subexpr.inverse().normal()) && subexprs.len() == 2)
                    .unwrap_or(false)
            },
            _ => false,
        }
    }

    fn children(&self) -> Set<Box<Self>> {
        log::debug!("[children] {self:?}");
        match self {
            Expr::And(children) | Expr::Or(children) => children.clone(),
            Expr::Not(expr) => Set::from([expr.clone()]),
            _ => Set::new(),
        }
    }

    fn dim_bound(&self) -> usize {
        log::debug!("[dim-bound] {self:?}");
        self.names().len() + 1
    }

    fn spawn(&self) -> SSet<Self> {
        log::debug!("[spawn] {self:?}");
        let atoms = self.atoms();
        atoms
            .iter()
            .filter_map(|&atom| {
                let axiom_guess = atom.axiom_set();
                if axiom_guess.iter().all(|atom| atoms.contains(atom)) {
                    Some(axiom_guess)
                } else {
                    None
                }
            })
            .collect()
    }

    fn fire(&self, tokens: SSet<Self>) -> SSet<Self> {
        log::debug!("[fire] {self:?} with {tokens:?}");
        let tokens_clone = tokens.clone();
        tokens_clone.iter()
            .flat_map(|token| token.iter()
                .map(move |expr| (token, expr)))
            .flat_map(|(token, expr)| {
                self.lineaged_subexprs().to_owned().iter()
                    .filter_map(move |lineage| {
                        if *lineage[0] == *expr {
                            Some((token.to_owned(), expr.to_owned(), lineage.to_owned()))
                        } else {
                            None
                        }
                    })
                    .collect::<Set<(Set<Self>, Expr, Vec<&Expr>)>>()
            })
            .filter_map(|(token, expr, lineage)| {
                log::trace!("[fire] token {token:?} check expr {expr:?} with lineage {lineage:?}");
                if lineage.len() <= 1 {
                    return None;
                }
                let mut partial_token = token.to_owned();
                partial_token.remove(&expr);
                let parent = lineage[1];

                let siblings = match parent {
                    Expr::And(exprs) | Expr::Or(exprs) => exprs,
                    _ => panic!("Expression {expr:?} has lineage {lineage:?}, but parent {parent:?} has no children!"),
                };
                let sibling_predicates = siblings.iter()
                    .map(|sibling| {
                        let mut partial_sibling = partial_token.to_owned();
                        partial_sibling.insert(*sibling.to_owned());
                        tokens_clone.contains(&partial_sibling)
                    })
                    .collect::<Vec<bool>>();

                let mut firing = partial_token.to_owned();
                firing.insert(parent.to_owned());
                log::trace!("[fire] firing predicates parent {parent:?} {}", sibling_predicates.len());
                match (parent, sibling_predicates.len()) {
                    (Expr::And(_), x) if x == siblings.len() => Some(firing),
                    (Expr::Or(_), x) if x > 0 => Some(firing),
                    _ => None,
                }
            })
            .chain(tokens)
            .collect()
    }

    fn project(&self, tokens: SSet<Self>) -> SSet<Self> {
        log::debug!("[project] {self:?} with {tokens:?}");
        tokens
            .iter()
            .flat_map(|token| {
                self.subexprs()
                    .iter()
                    .map(|&subexpr| {
                        let mut projection = token.to_owned();
                        projection.insert(subexpr.to_owned());
                        projection
                    })
                    .collect::<SSet<Self>>()
            })
            .collect()
    }
}
