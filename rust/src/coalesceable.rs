use btree_dag::{AddEdge, AddVertex};

use crate::{expression::Expr, Dag, Set};

pub trait Coalesceable: Sized + Ord + Clone + std::fmt::Debug {
    fn axiom_set(&self) -> Set<Self>;

    fn children(&self) -> Set<Box<Self>>;

    fn dim_bound(&self) -> usize;

    fn spawn(&self, proof: &mut Dag<Set<Self>>) -> Set<Set<Self>>;

    fn fire(&self, proof: &mut Dag<Set<Self>>, tokens: &Set<Set<Self>>) -> Set<Set<Self>>;

    fn project(&self, proof: &mut Dag<Set<Self>>, tokens: &Set<Set<Self>>) -> Set<Set<Self>>;

    fn coalesce(&self) -> Option<(Set<Set<Self>>, Dag<Set<Self>>)> {
        log::trace!("[coalesce] {self:?}");
        let mut proof = Dag::<Set<Self>>::new();

        let mut tokens = Self::spawn(self, &mut proof);
        if tokens.is_empty() {
            return None;
        }

        let mut old_tokens = Set::new();
        while !tokens.contains(&Set::from([self.clone()])) {
            log::trace!("[coalesce] {self:?} not in {tokens:?}");
            if old_tokens == tokens {
                let current_dim = tokens.iter().map(Set::len).fold(0, |a, b| a.max(b));
                if current_dim <= Self::dim_bound(self) {
                    tokens = Self::project(self, &mut proof, &tokens);
                } else {
                    return None;
                }
            }
            old_tokens = tokens.clone();
            tokens = Self::fire(self, &mut proof, &tokens);
        }

        Some((tokens, proof))
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

    fn spawn(&self, proof: &mut Dag<Set<Self>>) -> Set<Set<Self>> {
        log::trace!("[spawn] {self:?}");
        let atoms = self.atoms();
        atoms
            .iter()
            .filter_map(|&atom| {
                let axiom = atom.axiom_set();
                if axiom.iter().all(|atom| atoms.contains(atom)) {
                    log::debug!("∅ =T> {axiom:?}");
                    proof.add_vertex(axiom.clone());
                    Some(axiom)
                } else {
                    None
                }
            })
            .collect()
    }

    fn fire(&self, proof: &mut Dag<Set<Self>>, old_tokens: &Set<Set<Self>>) -> Set<Set<Self>> {
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
                        .collect::<Set<_>>(),
                    _ => panic!("Expression {expr:?} has lineage {lineage:?}, but parent {parent_expr:?} has no children!"),
                };

                log::trace!("[fire] given transition {expr:?} to {parent_expr:?}");
                log::trace!("[fire] is transition {token:?} to {parent_token:?} proveable");
                log::trace!("[fire] given subset {children:?} of {tokens:?}");

                // Is the parent operator satisfied appropriately for its children?
                match parent_expr {
                    Expr::And(_) if children.is_subset(&tokens)  => {
                        log::debug!("{children:?} =&> {parent_token:?}");
                        proof.add_vertex(parent_token.clone());
                        children.iter().for_each(|child| {
                            proof.add_vertex(child.clone());
                            proof.add_edge(child.clone(), parent_token.clone()).unwrap();
                        });
                        Some(parent_token)
                    },
                    Expr::Or(_) if !children.is_disjoint(&tokens) => {
                        log::debug!("{children:?} =|> {parent_token:?}");
                        proof.add_vertex(parent_token.clone());
                        proof.add_edge(token.clone(), parent_token.clone()).unwrap();
                        Some(parent_token)
                    },
                    _ => None,
                }
            })
            .chain(old_tokens.clone())
            .collect()
    }

    fn project(&self, proof: &mut Dag<Set<Self>>, tokens: &Set<Set<Self>>) -> Set<Set<Self>> {
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
                            proof.add_vertex(projection.clone());
                            proof.add_edge(token.clone(), projection.clone()).unwrap();
                            Some(projection)
                        } else {
                            None
                        }
                    })
                    .collect::<Set<_>>()
            })
            .collect()
    }
}
