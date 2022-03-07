#![feature(
    generic_associated_types,
    associated_type_defaults,
    generic_arg_infer,
    map_first_last
)]

use std::collections::{BTreeMap, BTreeSet};

pub mod coalesceable;
pub mod expression;
pub mod parseable;
pub mod proveable;

pub type Set<T> = BTreeSet<T>;
pub type SSet<T> = Set<Set<T>>;
pub type Map<K, V> = BTreeMap<K, V>;
pub type Node = usize;
pub type Edge = usize;
