#![feature(
    generic_associated_types,
    associated_type_defaults,
    generic_arg_infer,
    map_first_last
)]

use std::collections::{BTreeMap, BTreeSet};

use btree_dag::BTreeDag;

pub mod coalesceable;
pub mod expression;
pub mod parseable;

pub type Set<T> = BTreeSet<T>;
pub type Map<K, V> = BTreeMap<K, V>;
pub type Dag<T> = BTreeDag<T>;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Node(usize);
impl From<usize> for Node {
    fn from(i: usize) -> Self {
        Self(i)
    }
}
impl std::ops::AddAssign<usize> for Node {
    fn add_assign(&mut self, rhs: usize) {
        self.0 += rhs
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Edge(usize);
impl From<usize> for Edge {
    fn from(i: usize) -> Self {
        Self(i)
    }
}
impl std::ops::AddAssign<usize> for Edge {
    fn add_assign(&mut self, rhs: usize) {
        self.0 += rhs
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Arena<'arena, T> {
    arena: &'arena Vec<T>,
    index: usize,
}
impl<T: std::fmt::Debug> std::fmt::Debug for Arena<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.arena.get(self.index).unwrap().fmt(f)
    }
}
