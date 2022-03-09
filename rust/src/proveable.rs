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

impl<U: Coalesceable> Proof<Set<U>> {
    pub fn new(root: &U) -> Self {
        Self {
            root: Set::from([root.clone()]),
            node_idx: 0,
            nodes: Map::new(),
            edge_idx: 0,
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
                (&(from, to), _) if from == self.node_idx => Some(to),
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
                .is_axiom();
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
        self.backtrack_dag(&self.root.clone(), tokens);
    }

    fn backtrack_dag(&mut self, place: &Set<U>, tokens: &SSet<U>) {
        log::trace!("[backtrack] at place {place:?} with tokens {tokens:?}");
        tokens
            .iter()
            .filter(|&sub_place| Self::is_edge(&place, sub_place))
            .for_each(|sub_place| {
                let start = self.node(place);
                let end = self.node(sub_place);
                if let Some(_edge) = self.edge(&(start, end)) {
                    log::trace!("[backtrack] from {place:?} to {sub_place:?}");
                    self.backtrack_dag(sub_place, tokens);
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
        self.edges.get(edge).map(|_| None).unwrap_or_else(|| {
            let idx = self.edge_idx;
            self.edges.insert(edge.clone(), idx);
            self.edge_idx += 1;
            Some(idx)
        })
    }

    fn is_edge(start: &Set<U>, end: &Set<U>) -> bool {
        log::trace!("[is-edge] from {start:?} to {end:?}");
        // Weakening: S => S, p
        if start.is_superset(end) {
            log::trace!("[is-edge] is edge by weakening rule");
            return true;
        }

        let parent_diff = start.difference(end).collect::<Set<&U>>();
        let child_diff = end.difference(start).collect::<Set<&U>>();

        // BinaryOp: S, c => S, p  <=>  c in p
        if parent_diff.len() == 1 && child_diff.len() == 1 {
            let &parent = parent_diff.first().unwrap();
            let &child = child_diff.first().unwrap();
            if parent.children().get(child).is_some() {
                log::trace!("[is-edge] is edge by binary-operator rule");
                return true;
            }
        }
        // Contraction: S, c => S  <=>  p in S : c in p
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
