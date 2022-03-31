#![feature(
    generic_associated_types,
    associated_type_defaults,
    generic_arg_infer,
    map_first_last
)]

use std::{
    collections::{BTreeMap, BTreeSet},
};

use btree_dag::{BTreeDag};

pub type Set<T> = BTreeSet<T>;
pub type Map<K, V> = BTreeMap<K, V>;
pub type Dag<T> = BTreeDag<T>;

pub mod coalesceable;
pub mod expression;
pub mod parseable;