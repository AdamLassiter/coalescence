#! /usr/bin/env python3

from __future__ import annotations
from collections import defaultdict
from dataclasses import dataclass, field
from typing import Optional

from coalescence import coalescence, Place
from expression import Expr
from parser import parse
from utils import unwrap


def proof(expr: Expr) -> Optional[ProofDAG]:
    coalesced = coalescence(expr)
    if coalesced is not None:
        proof_ = _backtrack_dag(frozenset({expr}), coalesced)
        if proof_ is not None:
            return proof_
        else:
            raise Exception('Proveable %s but failed to construct backtrack' % expr)
    else:
        raise Exception('Not proveable')


@dataclass
class ProofDAG:
    root: Place
    edges: dict[Place, set[Place]] = field(default_factory = lambda: defaultdict(set))

    def insert(self, start: Place, end: Place) -> bool:
        if end not in self.edges[start]:
            self.edges[start].add(end)
            return True
        else:
            return False


    def graph(self):
        try:
            import graphviz # type: ignore
            from itertools import chain
        except ImportError:
            return
        dag = graphviz.Digraph(comment='Proof of %s' % self.root)
        for node in set(chain(self.edges.keys(), *self.edges.values())):
            dag.node('%s' % hash(node), str(','.join(map(str, node))))
        for start, edges in self.edges.items():
            for end in edges:
                dag.edge('%s' % hash(start), '%s' % hash(end))
        dag.render('%s.gv' % abs(hash(self.root)), view=True)


def _is_edge(start: Place, end: Place) -> bool:
    # Weakening: S => S, p
    if start > end:
        return True
    parent_diff = start - end
    child_diff = end - start
    # BinaryOp: S, c => S, p  <=>  c in p
    if len(parent_diff) == 1 and len(child_diff) == 1:
        if unwrap(child_diff) in unwrap(parent_diff).exprs:
            return True
    # Contraction: S, c => S  <=>  p in S : c in p
    if len(child_diff) == 1 and len(parent_diff) <= 1:
        if any(map(lambda x: unwrap(child_diff) in x.exprs, start)):
            return True
    return False


def _backtrack_dag(place: Place, tokens: frozenset[Place], dag: Optional[ProofDAG] = None) -> ProofDAG:
    if not dag:
        dag = ProofDAG(place)
    for sub_place in filter(lambda x: _is_edge(place, x), tokens):
        if dag.insert(place, sub_place):
            _backtrack_dag(sub_place, tokens, dag=dag)
    return dag


if __name__ == '__main__':
    import readline
    while True:
        proof_ = proof(parse(input('>> ')).normalize())
        if proof_ is not None:
            proof_.graph()
