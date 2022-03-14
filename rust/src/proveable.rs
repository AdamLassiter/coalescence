use crate::{coalesceable::Coalesceable, expression::Expr, Edge, Map, Node, SSet, Set};

#[derive(Debug)]
pub struct Proof<T> {
    root: T,
    node_idx: Node,
    nodes: Map<T, Node>,
    edge_idx: Edge,
    edges: Map<(Node, Node), Edge>,
}

pub trait Proveable: Coalesceable {
    fn axiom(&self) -> Set<Self>;

    fn proof(&self) -> Result<Proof<Set<Self>>, String>;
}

impl Proveable for Expr {
    fn axiom(&self) -> Set<Self> {
        Set::from([self.clone(), self.inverse().normal()])
    }

    fn proof(&self) -> Result<Proof<Set<Self>>, String>
    where
        Self: Coalesceable,
    {
        log::trace!("[proof] {self:?}");
        let coalesced = self.coalesce().ok_or("Not coalesceable")?;

        let mut proof = Proof::new(self);
        proof.backtrack(&coalesced);

        Ok(proof)
    }
}

impl<U: Coalesceable + std::fmt::Debug> Proof<Set<U>> {
    pub fn new(root: &U) -> Self {
        Self {
            root: Set::from([root.clone()]),
            node_idx: 0.into(),
            nodes: Map::new(),
            edge_idx: 0.into(),
            edges: Map::new(),
        }
    }

    pub fn verify(&self) -> Result<(), String> {
        log::trace!("[verify] {self:?}");
        let &root_node = self
            .nodes
            .get(&self.root)
            .ok_or("Root node is not in nodes-map")?;
        self.walk_to_axiom(root_node)
    }

    fn walk_to_axiom(&self, node: Node) -> Result<(), String> {
        log::trace!("[walk-to-axiom] walked to {node:?}");

        let edges = self
            .edges
            .iter()
            .filter_map(|edge| match edge {
                (&(from, to), _) if from == node => Some(to),
                _ => None,
            })
            .collect::<Vec<_>>();

        if edges.is_empty() {
            log::trace!("[walk-to-axiom] edges empty at {node:?}");

            let place = self
                .nodes
                .iter()
                .find_map(|(key, &val)| if val == node { Some(key) } else { None })
                .ok_or(format!("Node index {node:?} is not in nodes-map"))?;

            let is_axiom = place
                .first()
                .ok_or("Node index {node:?} is an empty place")?
                .axiom_set()
                .eq(place);

            if is_axiom {
                Ok(())
            } else {
                Err(format!(
                    "Node {place:?} not an axiom but terminates a proof tree"
                ))
            }
        } else {
            log::trace!("[walk-to-axiom] traversing edges {edges:?} at {node:?}");

            edges
                .iter()
                .map(|&next| self.walk_to_axiom(next))
                .fold(Ok(()), |acc, elem| match (acc, elem) {
                    (Err(left), Err(right)) => Err(format!("{left}\n{right}")),
                    (Err(left), Ok(())) => Err(left),
                    (Ok(()), Err(right)) => Err(right),
                    (Ok(()), Ok(())) => Ok(()),
                })
        }
    }

    fn backtrack(&mut self, tokens: &SSet<U>) {
        log::trace!("[backtrack] {tokens:?}");
        self.backtrack_dag(vec![self.root.clone()], tokens);
    }

    // TODO: Don't produce acyclic graphs, probably by tracking depth
    fn backtrack_dag(&mut self, path: Vec<Set<U>>, tokens: &SSet<U>) {
        let place = path.last().unwrap();
        log::trace!("[backtrack] at place {place:?} with tokens {tokens:?}");

        tokens
            .iter()
            .filter(|&next_place| !path.contains(next_place))
            .filter(|&next_place| Self::is_edge(&place, next_place))
            .for_each(|next_place| {
                let start = self.node(place);
                let end = self.node(next_place);
                if self.edge(&(start, end)).is_some() {
                    log::trace!("[backtrack] from {place:?} to {next_place:?}");
                    let next_path = {
                        let mut partial = path.clone();
                        partial.push(next_place.to_owned());
                        partial
                    };
                    self.backtrack_dag(next_path, tokens);
                }
            });
    }

    fn node(&mut self, node: &Set<U>) -> Node {
        log::trace!("[node] {node:?}");
        self.nodes.get(&node).map(|&x| x).unwrap_or_else(|| {
            let idx = self.node_idx;
            self.nodes.insert(node.clone(), idx);
            self.node_idx += 1;
            idx
        })
    }

    fn edge(&mut self, edge: &(Node, Node)) -> Option<Edge> {
        log::trace!("[edge] {edge:?}");
        self.edges
            .get(edge)
            .map(|idx| {
                log::trace!("[edge] already inserted at {idx:?}");
                None
            })
            .unwrap_or_else(|| {
                self.edges
                    .get(&(edge.1, edge.0))
                    .map(|idx| {
                        log::trace!("[edge] reverse already inserted at {idx:?}");
                        None
                    })
                    .unwrap_or_else(|| {
                        let idx = self.edge_idx;
                        log::trace!("[edge] new insert at {idx:?}");
                        self.edges.insert(edge.clone(), idx);
                        self.edge_idx += 1;
                        Some(idx)
                    })
            })
    }

    fn is_edge(start: &Set<U>, end: &Set<U>) -> bool {
        log::trace!("[is-edge] from {start:?} to {end:?}");

        // Weakening: S => S, p
        if start.is_superset(end) && !start.is_subset(end) {
            log::trace!("[is-edge] is edge by weakening rule");
            return true;
        }

        let parent_diff = start.difference(end).collect::<Set<&U>>();
        let child_diff = end.difference(start).collect::<Set<&U>>();

        // BinaryOp: S, c => S, p  <=>  c -> p
        if parent_diff.len() == 1 && child_diff.len() == 1 {
            let &parent = parent_diff.first().unwrap();
            let &child = child_diff.first().unwrap();
            if parent.children().get(child).is_some() {
                log::trace!("[is-edge] is edge by binary-operator rule");
                return true;
            }
        }

        // Contraction: S, c => S  <=>  exists p in S : c -> p
        // [Degenerate case of BinaryOp]
        if parent_diff.len() <= 1 && child_diff.len() == 1 {
            let &child = child_diff.first().unwrap();
            if start
                .iter()
                .filter(|&sub_parent| sub_parent.children().contains(child))
                .next()
                .is_some()
            {
                log::trace!("[is-edge] is edge by contraction rule");
                return true;
            }
        }

        log::trace!("[is-edge] not edge");
        false
    }
}
