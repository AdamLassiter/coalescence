#![feature(
    generic_associated_types,
    associated_type_defaults,
    generic_arg_infer,
    map_first_last
)]

use std::collections::{BTreeMap, BTreeSet};

pub mod coalescence;
pub mod expression;
pub mod parser;
pub mod proof;

pub(crate) type Set<T> = BTreeSet<T>;
pub(crate) type SSet<T> = Set<Set<T>>;
pub(crate) type Map<K, V> = BTreeMap<K, V>;
pub(crate) type Node = usize;
pub(crate) type Edge = usize;

#[allow(dead_code)]
pub (crate) fn log_init() {
    let _ = env_logger::builder()
        .is_test(true)
        .try_init();
}
