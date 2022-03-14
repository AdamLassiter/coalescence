use crate::{expression::Expr, SSet, Set};

pub trait Coalesceable: Sized + Ord + Clone + std::fmt::Debug {
    fn axiom_set(&self) -> Set<Self>;

    fn children(&self) -> Set<Box<Self>>;

    fn dim_bound(&self) -> usize;

    fn spawn(&self) -> SSet<Self>;

    fn fire(&self, tokens: &SSet<Self>) -> SSet<Self>;
    fn sparse_fire(&self, tokens: &SSet<Self>) -> SSet<Self>;

    fn project(&self, tokens: &SSet<Self>) -> SSet<Self>;
    fn sparse_project(&self, tokens: &SSet<Self>) -> SSet<Self>;

    fn coalesce(&self) -> Option<SSet<Self>> {
        log::trace!("[coalesce] {self:?}");
        let mut tokens = Self::spawn(self);
        if tokens.is_empty() {
            return None;
        }

        let mut old_tokens = Set::new();
        while !tokens.contains(&Set::from([self.clone()])) {
            log::trace!("[coalesce] {self:?} not in {tokens:?}");
            if old_tokens == tokens {
                let current_dim = tokens.iter().map(Set::len).fold(0, |a, b| a.max(b));
                if current_dim <= Self::dim_bound(self) {
                    tokens = Self::sparse_project(self, &tokens);
                } else {
                    return None;
                }
            }
            old_tokens = tokens.clone();
            tokens = Self::sparse_fire(self, &tokens);
        }

        Some(tokens)
    }
}

impl Coalesceable for Expr {
    fn axiom_set(&self) -> Set<Self> {
        Set::from([self.clone(), self.inverse().normal()])
    }

    fn children(&self) -> Set<Box<Self>> {
        log::trace!("[children] {self:?}");
        match self {
            Expr::And(children) | Expr::Or(children) => children.clone(),
            Expr::Not(expr) => Set::from([expr.clone()]),
            _ => Set::new(),
        }
    }

    fn dim_bound(&self) -> usize {
        log::trace!("[dim-bound] {self:?}");
        self.atoms().len()
    }

    fn spawn(&self) -> SSet<Self> {
        log::trace!("[spawn] {self:?}");
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
            .inspect(|axiom| log::debug!("∅ =T> {axiom:?}"))
            .collect()
    }

    fn fire(&self, old_tokens: &SSet<Self>) -> SSet<Self> {
        log::trace!("[fire] {self:?} with {old_tokens:?}");
        let tokens = old_tokens.clone();
        tokens.iter()
            .flat_map(|token| token.iter()
                .map(move |expr| (token, expr)))
            // TODO: This could be... better in some way?
            .flat_map(|(token, expr)| {
                self.lineage().to_owned().iter()
                    .filter_map(move |lineage| {
                        if *lineage[0] == *expr {
                            Some((token.to_owned(), expr.to_owned(), lineage.to_owned()))
                        } else {
                            None
                        }
                    })
                    .collect::<Set<_>>()
            })
            .filter_map(|(token, expr, lineage)| {
                log::trace!("[fire] token {token:?} inspecting {expr:?} with lineage {lineage:?}");
                // Is this the root?
                if lineage.len() <= 1 {
                    return None;
                }

                // If not the root, then there exists a parent of this expression
                let parent_expr = lineage[1];

                // There's a constructor of sibling places
                let sibling_token_fn = |sibling| {
                    let mut partial = token.to_owned();
                    partial.remove(&expr);
                    partial.insert(sibling);
                    partial
                };
                // There's also a place for the parent
                let parent_token = sibling_token_fn(parent_expr.to_owned());

                // Short-circuit if we have already deduced the parent
                if tokens.contains(&parent_token) {
                    return None;
                }

                // If there's a parent, then this expr has siblings
                let children = match parent_expr {
                    Expr::And(exprs) | Expr::Or(exprs) => exprs.iter()
                        .map(|sibling| sibling_token_fn(*sibling.to_owned()))
                        .collect::<SSet<_>>(),
                    _ => panic!("Expression {expr:?} has lineage {lineage:?}, but parent {parent_expr:?} has no children!"),
                };

                log::trace!("[fire] given transition {expr:?} to {parent_expr:?}");
                log::trace!("[fire] is transition {token:?} to {parent_token:?} proveable");
                log::trace!("[fire] given subset {children:?} of {tokens:?}");

                // Is the parent operator satisfied appropriately for its children?
                match parent_expr {
                    Expr::And(_) if children.is_subset(&tokens)  => {
                        log::debug!("{children:?} =&> {parent_token:?}");
                        Some(parent_token)
                    },
                    Expr::Or(_) if !children.is_disjoint(&tokens) => {
                        log::debug!("{children:?} =|> {parent_token:?}");
                        Some(parent_token)
                    },
                    _ => None,
                }
            })
            .chain(old_tokens.clone())
            .collect()
    }

    fn sparse_fire(&self, old_tokens: &SSet<Self>) -> SSet<Self> {
        log::trace!("[sparse-fire] {self:?} with {old_tokens:?}");
        let mut firing = self.fire(&old_tokens);
        old_tokens.iter().for_each(|token| {
            if token.iter().all(|expr| {
                let parent = self.lineage()
                    .iter()
                    .find_map(move |lineage| if lineage[0] == expr {
                        lineage.get(1).map(|x| x.clone())
                    } else {
                        None
                    });
                parent.map(|parent| firing.contains(token)
                    && firing.contains(&{
                        let mut partial = token.clone();
                        partial.insert(parent.clone());
                        partial
                    })
                    && firing.contains(&{
                        let mut partial = token.clone();
                        partial.remove(expr);
                        partial.insert(parent.clone());
                        partial
                    })).unwrap_or(false)
            }) {
                firing.remove(token);
            }
        });
        firing
    }

    fn project(&self, tokens: &SSet<Self>) -> SSet<Self> {
        log::trace!("[project] {self:?} with {tokens:?}");
        tokens
            .iter()
            .flat_map(|token| {
                self.subexprs()
                    .iter()
                    .filter_map(|&subexpr| {
                        let projection = {
                            let mut partial = token.to_owned();
                            partial.insert(subexpr.to_owned());
                            partial
                        };
                        if !tokens.contains(&projection) {
                            log::debug!("{token:?} =%> {projection:?}");
                            Some(projection)
                        } else {
                            None
                        }
                    })
                    .collect::<SSet<_>>()
            })
            .collect()
    }

    fn sparse_project(&self, tokens: &SSet<Self>) -> SSet<Self> {
        log::trace!("[sparse-project] {self:?} with {tokens:?}");
        self.project(tokens)
    }
}
