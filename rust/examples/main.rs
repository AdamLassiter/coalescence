use std::hash::Hash;
use std::{collections::hash_map::DefaultHasher, hash::Hasher};

use btree_dag::{Connections, Vertices};
use coalescence::{Dag, Set};

#[derive(Ord, PartialOrd, Eq, PartialEq, Clone, std::fmt::Debug)]
struct Nd<T>(Set<T>);

#[derive(Ord, PartialOrd, Eq, PartialEq, Clone, std::fmt::Debug)]
struct Ed<T>(Set<T>, Set<T>);

pub type Graph<T> = Dag<Set<T>>;

impl<'a, T> dot::Labeller<'a, Nd<T>, Ed<T>> for Graph<T>
where
    T: Ord + Hash + std::fmt::Debug,
{
    fn graph_id(&'a self) -> dot::Id<'a> {
        dot::Id::new("Proof").unwrap()
    }
    fn node_id(&'a self, n: &Nd<T>) -> dot::Id<'a> {
        let mut hasher = DefaultHasher::new();
        n.0.hash(&mut hasher);
        dot::Id::new(format!("N{}", hasher.finish())).unwrap()
    }

    fn node_label(&'a self, n: &Nd<T>) -> dot::LabelText<'a> {
        dot::LabelText::LabelStr(format!("{:?}", n.0).into())
    }
}

impl<'a, T: Ord + Clone> dot::GraphWalk<'a, Nd<T>, Ed<T>> for Graph<T> {
    fn nodes(&self) -> dot::Nodes<'a, Nd<T>> {
        self.vertices()
            .iter()
            .map(|&node| Nd(node.clone()))
            .collect()
    }
    fn edges(&'a self) -> dot::Edges<'a, Ed<T>> {
        self.vertices()
            .iter()
            .flat_map(|&node| {
                self.connections(node.clone())
                    .unwrap()
                    .iter()
                    .map(|target| Ed(node.clone(), target.clone()))
            })
            .collect()
    }
    fn source(&self, e: &Ed<T>) -> Nd<T> {
        Nd(e.0.clone())
    }
    fn target(&self, e: &Ed<T>) -> Nd<T> {
        Nd(e.1.clone())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    use coalescence::{coalesceable::Coalesceable, expression::Expr, parseable::Parseable};
    use std::fs::File;

    env_logger::init();

    while let Some(input) = rprompt::prompt_reply_stdout("ψ. ").ok() {
        let expr = Expr::parse(&input)?.normal();
        log::info!("Input: {expr:?}");
        let (_, proof) = expr.coalesce().ok_or("Not coalesceable")?;

        let graph: Graph<Expr> = proof;

        dot::render(&graph, &mut File::create("proof.dot").unwrap()).unwrap();
    }

    Ok(())
}
