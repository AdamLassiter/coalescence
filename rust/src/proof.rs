use crate::{coalescence::Coalesceable, expression::Expr, SSet, Set, Edge, Map, Node};

#[derive(Debug)]
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
            if parent.children()
                .get(child)
                .is_some() {
                return true;
            }
        }
        // Contraction: S, c => S  <=>  p in S : c in p
        if  parent_diff.len() <= 1 && child_diff.len() == 1 {
            let &child = child_diff.first().unwrap();
            if start.iter()
                .filter(|&sub_parent| sub_parent.children().contains(child))
                .next()
                .is_some() {
                return true;
            }
        }

        false
    }

    fn verify(&self) -> Result<(), String> {
        self.walk_to_axiom(&self.root)
    }

    fn walk_to_axiom(&self, node: Node) -> Result<(), String> {
        let edges = self.edges.iter()
            .filter_map(|edge| match edge {
                (from, to) if from == self.node_idx => Some(to),
                _ => None,
            })
            .collect();
        if edges.is_empty() {
            let place: [U] = self.nodes.iter()
                .find_map(|(key, val)| if val == node {Some(key)} else {None})
                .ok_or(format!("Node index {node:?} is not in nodes-map"))?
                .into();
            match place {
                [a, b] if a == b.inverse() => Ok(()),
                _ => Err("Node {place:?} not an axiom but terminates a  proof tree".to_string()),
            }
        } else {
            edges.iter()
                .map(|next| self.walk_to_axiom(next))
                .fold(Ok(()), |acc, elem: Result<(), String>| match (acc, elem) {
                    (Err(left), Err(right)) => Err(format!("{left}\n{right}")),
                    (Err(left), Ok(())) => Err(left),
                    (Ok(()), Err(right)) => Err(right),
                    (Ok(()), Ok(())) => Ok(()),
                })
        }
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
    fn test_proof_axiom() -> Result<(), String> {
        let expr = Expr::parse("a > a")?.normal();
        expr.proof()?;
        Ok(())
    }

    #[test]
    fn test_proof_duplicate_axiom() -> Result<(), String> {
        let expr = Expr::parse("(a > a) & (a > a)")?.normal();
        expr.proof()?;
        Ok(())
    }

    #[test]
    fn test_proof_two_axioms() -> Result<(), String> {
        let expr = Expr::parse("(a > a) & (b > b)")?.normal();
        expr.proof()?;
        Ok(())
    }

    #[test]
    fn test_proof_second_axiom() -> Result<(), String> {
        let expr = Expr::parse("(a & b) | (~a & b) | (a & ~b) | (~a & ~b)")?.normal();
        expr.proof()?;
        Ok(())
    }

    #[test]
    fn test_proof_second_axiom_invalid() -> Result<(), String> {
        let expr = Expr::parse("(a & b) | (~a & b) | (a & ~b)")?.normal();
        let _ = expr.proof().unwrap_err();
        Ok(())
    }

    #[test]
    fn test_proof_fourth_axiom() -> Result<(), String> {
        let expr = Expr::parse("(a & b & c & d) | (a & ~b & c & d) | (~a & b & c & d) | (~a & ~b & c & d) | (a & b & ~c & d) | (a & ~b & ~c & d) | (~a & b & ~c & d) | (~a & ~b & ~c & d) | (a & b & c & ~d) | (a & ~b & c & ~d) | (~a & b & c & ~d) | (~a & ~b & c & ~d) | (a & b & ~c & ~d) | (a & ~b & ~c & ~d) | (~a & b & ~c & ~d) | (~a & ~b & ~c & ~d)")?.normal();
        expr.proof()?;
        Ok(())
    }
}
