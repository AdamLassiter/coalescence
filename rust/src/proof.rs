use crate::{coalescence::Coalesceable, expression::Expr, SSet, Set, Edge, Map, Node};

pub struct Proof<T> {
    root: T,
    node_idx: Node,
    nodes: Map<T, Node>,
    edge_idx: Edge,
    edges: Map<(Node, Node), Edge>,
}

impl <U: Coalesceable> Proof<Set<U>> {
    pub fn new(root: &U) -> Self {
        Self {
            root: Set::from([root.clone()]),
            node_idx: 0,
            nodes: Map::new(),
            edge_idx: 0,
            edges: Map::new(),
        }
    }

    pub fn backtrack(&mut self, tokens: &SSet<U>) {
        self.backtrack_dag(&self.root.clone(), tokens);
    }

    fn backtrack_dag (&mut self, place: &Set<U>, tokens: &SSet<U>) {
        tokens
            .iter()
            .filter(|&sub_place| Self::is_edge(&place, sub_place))
            .for_each(|sub_place| {
                let start = self.node(place);
                let end = self.node(sub_place);
                if let Some(_edge) = self.edge(&(start, end)) {
                    self.backtrack_dag(sub_place, tokens);
                }
            });
    }

    fn node(&mut self, node: &Set<U>) -> Node {
        self.nodes.get(&node)
            .map(|&x| x)
            .unwrap_or_else(|| {
                let idx = self.node_idx;
                self.nodes.insert(node.clone(), idx);
                self.node_idx += 1;
                idx
            })
    }

    fn edge(&mut self, edge: &(Node, Node)) -> Option<Edge> {
        self.edges.get(edge)
            .map(|_| None)
            .unwrap_or_else(|| {
                let idx = self.edge_idx;
                self.edges.insert(edge.clone(), idx);
                self.edge_idx += 1;
                Some(idx)
            })
    }

    fn is_edge(start: &Set<U>, end: &Set<U>) -> bool {
        // Weakening: S => S, p
        if start.is_superset(end) {
            return true;
        }

        let parent_diff = start.difference(end).collect::<Set<&U>>();
        let child_diff = end.difference(start).collect::<Set<&U>>();

        // BinaryOp: S, c => S, p  <=>  c in p
        if  parent_diff.len() == 1 && child_diff.len() == 1 {
            let &parent = parent_diff.first().unwrap();
            let &child =  child_diff.first().unwrap();
            if let Some(_) = parent.children().get(child) {
                return true;
            }
        }
        // Contraction: S, c => S  <=>  p in S : c in p
        if  parent_diff.len() <= 1 && child_diff.len() == 1 {
            let &child = child_diff.first().unwrap();
            if let Some(_) = start.iter()
                .filter(|&sub_parent| sub_parent.children().contains(child))
                .next() {
                return true;
            }
        }
        
        false
    }

    fn verify(&self) -> Result<(), String> {
        Ok(())
    }
}

pub trait Proveable: Coalesceable {
    fn proof(&self) -> Result<Proof<Set<Self>>, String>;
}

impl Proveable for Expr {
    fn proof(&self) -> Result<Proof<Set<Self>>, String> where Self: Coalesceable {
        let coalesced = self.coalesce().ok_or("Not coalesceable")?;

        let mut proof = Proof::new(self);
        proof.backtrack_dag(&Set::from([self.clone()]), &coalesced);

        proof.verify().map(|_| proof)
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::Parseable;

    use super::*;

    #[test]
    fn test_axiom() {
        let expr = Expr::parse("a > a").unwrap().normal();
        assert!(expr.proof().is_ok());
    }
}