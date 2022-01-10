#! /usr/bin/env python3
# pylint: disable=W0640,W0611,W0703

from __future__ import annotations
from dataclasses import dataclass
from typing import Set, FrozenSet, Optional, Tuple, Iterable, cast

from expression import Expr, And, Or, Atom, NotAtom
from parser import parse
from utils import flat_map, peek_any, loop_into_none


@dataclass(frozen = True)
class ProofTree:
    sequent: FrozenSet[Expr]
    subproofs: FrozenSet[ProofTree]

    def __str__(self) -> str:
        return '\n'.join(self.pretty() + ['q.e.d'])

    def pretty(self) -> list[str]:
        return ['|\t' * i + line \
                for i, subproof in enumerate(self.subproofs)
                for line in subproof.pretty()] + \
            [len(self.subproofs) * '+-------',
             '|- ' + ', '.join(map(str, self.sequent))]


def spawn(expr: Expr) -> FrozenSet[FrozenSet[Expr]]:
    atoms = expr.atoms()
    spawns: Set[FrozenSet[Expr]] = set()
    for atom in atoms:
        not_atom = (~atom).normalize()
        if not_atom in atoms:
            spawns |= {frozenset({atom, not_atom})}
    return frozenset(spawns)


def fire(expr: Expr, tokens: FrozenSet[FrozenSet[Expr]]) -> FrozenSet[FrozenSet[Expr]]:
    next_tokens = set(tokens)
    for token in tokens:
        for sub_expr in token:
            partial_token = token - {sub_expr}
            for lineage in filter(lambda x: x[0] == sub_expr, expr.lineaged_subexprs()):
                if len(lineage) > 1:
                    parent = lineage[1]
                    # print('Lineage:', lineage)
                    pred_and = all(map(lambda x: {x} | partial_token in tokens, parent.exprs))
                    pred_or = any(map(lambda x: {x} | partial_token in tokens, parent.exprs))
                    if (type(parent) == And and pred_and) or \
                            (type(parent) == Or and pred_or):
                        next_tokens |= {frozenset({parent} | partial_token)}
    return frozenset(next_tokens)


def project(expr: Expr, tokens: FrozenSet[FrozenSet[Expr]]) -> FrozenSet[FrozenSet[Expr]]:
    projection: Set[FrozenSet[Expr]] = set()
    for token in tokens:
        for atom in expr.subexprs():
            # print('Project:', list(token), atom)
            projection |= {frozenset(token | {atom})}
    return frozenset(projection)


def coalesce(expr: Expr) -> Optional[FrozenSet[FrozenSet[Expr]]]:
    tokens = spawn(expr)
    # print('Spawn:', *map(list, tokens), sep='\n\t')
    if not tokens:
        # print('Not Spawnable')
        return None
    old_tokens = None
    max_dim = len(set(map(lambda x: peek_any(x.exprs), expr.atoms())))
    while {expr} not in tokens:
        cur_dim = max(map(len, tokens))
        if old_tokens == tokens:
            if cur_dim <= max_dim:
                tokens = project(expr, tokens)
                # print('Project:', *map(list, tokens), sep='\n\t')
            else:
                # print('Not Proveable')
                return None
        old_tokens = tokens
        tokens = fire(expr, old_tokens)
        # print('Fire:', *map(list, tokens), sep='\n\t')
    # print('Proveable:', *map(list, tokens), sep='\n\t')
    return tokens


@loop_into_none
def backtrack(place: FrozenSet[Expr], tokens: FrozenSet[FrozenSet[Expr]]) -> Optional[ProofTree]:
    for subexpr in place:
        not_subexpr = Expr.Not(subexpr).normalize()
        if isinstance(subexpr, Atom):
            if not_subexpr in place:
                return ProofTree(frozenset({subexpr, not_subexpr}), frozenset())
            else:
                continue
        narrow_pass = lambda p, s, e: (p | {s}) - {e}
        wide_pass = lambda p, s, e: p | {s}
        for pass_fn in [narrow_pass, wide_pass]:
            maybe_subproofs = frozenset(map(lambda x: backtrack(pass_fn(place, x, subexpr), tokens), subexpr.subexprs))
            subproofs = maybe_subproofs - {None}
            if type(subexpr) == Or and subproofs:
                return ProofTree(place, frozenset({peek_any(subproofs)}))
            elif type(subexpr) == And and subproofs == maybe_subproofs:
                return ProofTree(place, frozenset(cast(Iterable[ProofTree], subproofs)))
    return None


def prove(expr: Expr) -> Optional[ProofTree]:
    coalescence = coalesce(expr)
    if coalescence is not None:
        proof = backtrack(frozenset({expr}), coalescence)
        if proof is not None:
            return proof
        else:
            raise Exception('Proveable %s but failed to construct backtrack' % expr)
    else:
        return None


if __name__ == '__main__':
    import readline
    while True:
        input_expr = parse(input('>> ')).normalize()
        expr_proof = prove(input_expr)
        print('Proof:', expr_proof)
