#! /usr/bin/env python3
# pylint: disable=W0640,W0611,W0703

from __future__ import annotations
from dataclasses import dataclass
from itertools import product
from typing import Optional, Generator

from expression import Expr, And, Or, Atom
from parser import parse
from utils import flat_map, peek, unwrap, unwrap_any, maybe_unwrap_any


@dataclass(frozen=True)
class ProofTree:
    sequent: frozenset[Expr]
    subproofs: frozenset[ProofTree]

    def __str__(self) -> str:
        return '\n'.join(self.pretty() + ['q.e.d'])

    def pretty(self) -> list[str]:
        return ['|\t' * i + line
                for i, subproof in enumerate(self.subproofs)
                for line in subproof.pretty()] + \
            [len(self.subproofs) * '+-------',
             '|- ' + ', '.join(map(str, self.sequent))]


def spawn(expr: Expr) -> frozenset[frozenset[Expr]]:
    atoms = expr.atoms()
    spawns: set[frozenset[Expr]] = set()
    for atom in atoms:
        not_atom = (~atom).normalize()
        if not_atom in atoms:
            spawns |= {frozenset({atom, not_atom})}
    return frozenset(spawns)


def fire(expr: Expr, tokens: frozenset[frozenset[Expr]]) -> frozenset[frozenset[Expr]]:
    next_tokens = set(tokens)
    for token in tokens:
        for sub_expr in token:
            partial_token = token - {sub_expr}
            for lineage in filter(lambda x: x[0] == sub_expr, expr.lineaged_subexprs()):
                if len(lineage) > 1:
                    parent = lineage[1]
                    pred_and = all(
                        map(lambda x: {x} | partial_token in tokens, parent.exprs))
                    pred_or = any(
                        map(lambda x: {x} | partial_token in tokens, parent.exprs))
                    if (type(parent) == And and pred_and) or \
                            (type(parent) == Or and pred_or):
                        next_tokens |= {frozenset({parent} | partial_token)}
    return frozenset(next_tokens)


def project(expr: Expr, tokens: frozenset[frozenset[Expr]]) -> frozenset[frozenset[Expr]]:
    projection: set[frozenset[Expr]] = set()
    for token in tokens:
        for atom in expr.subexprs():
            projection |= {frozenset(token | {atom})}
    return frozenset(projection)


def coalesce(expr: Expr) -> Optional[frozenset[frozenset[Expr]]]:
    tokens = spawn(expr)
    if not tokens:
        return None
    old_tokens = None
    max_dim = len(set(map(lambda x: unwrap(x.exprs), expr.atoms())))
    while {expr} not in tokens:
        cur_dim = max(map(len, tokens))
        if old_tokens == tokens:
            if cur_dim <= max_dim:
                tokens = project(expr, tokens)
            else:
                return None
        old_tokens = tokens
        tokens = fire(expr, old_tokens)
    return tokens


def backtrack(place: frozenset[Expr], tokens: frozenset[frozenset[Expr]], lineage: frozenset[frozenset[Expr]] = frozenset()) -> Generator[ProofTree, None, None]:
    if place not in tokens:
        return

    if place in lineage:
        return

    for subexpr in place:
        if isinstance(subexpr, Atom):
            axiom = frozenset({subexpr, Expr.Not(subexpr).normalize()})
            if axiom.issubset(place) and axiom in tokens:
                # => S, a, ~a
                yield ProofTree(place, frozenset({ProofTree(axiom, frozenset())}))
                return

        for subplaces_fn in [lambda x: (place | {x}) - {subexpr},
                             lambda x: (place | {x})]:
            any_subplaces = filter(lambda x: x in tokens, map(subplaces_fn, subexpr.subexprs()))
            if type(subexpr) == Or:
                # S, a => S, a || b
                subproofs = frozenset(map(lambda x: backtrack(x, tokens, lineage | {place}), any_subplaces))
                if any(subproofs):
                    yield ProofTree(place, frozenset(flat_map(lambda x: x, subproofs)))
                    return

            all_subplaces: filter[frozenset[Expr]] = filter(lambda x: x in tokens, map(frozenset, product(*any_subplaces)))
            if type(subexpr) == And:
                # S, a   S, b => S, a && b
                subproofs = frozenset(map(lambda x: backtrack(x, tokens, lineage | {place}), all_subplaces))
                if all(subproofs):
                    yield ProofTree(place, frozenset(flat_map(lambda x: x, subproofs)))
                    return


def prove(expr: Expr) -> Optional[ProofTree]:
    coalescence = coalesce(expr)
    if coalescence is not None:
        proofs = frozenset(backtrack(frozenset({expr}), coalescence))
        if proofs:
            proof = unwrap_any(proofs)
            print('Proof:', proof)
            return proof
        else:
            raise Exception('Proveable %s but failed to construct backtrack' % expr)
    else:
        return None


if __name__ == '__main__':
    import readline
    while True:
        prove(parse(input('>> ')).normalize())
