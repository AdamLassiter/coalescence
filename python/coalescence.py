#! /usr/bin/env python3

from __future__ import annotations
from typing import Optional

from expression import Expr, And, Or
from utils import unwrap


Place = frozenset[Expr]


def coalescence(expr: Expr) -> Optional[frozenset[Place]]:
    tokens = _spawn(expr)
    if not tokens:
        return None
    old_tokens = None
    max_dim = len(set(map(lambda x: unwrap(x.exprs), expr.atoms()))) + 1
    while {expr} not in tokens:
        cur_dim = max(map(len, tokens))
        if old_tokens == tokens:
            if cur_dim < max_dim:
                tokens = _project(expr, tokens)
            else:
                return None
        old_tokens = tokens
        tokens = _fire(expr, old_tokens)
    return tokens


def _spawn(expr: Expr) -> frozenset[Place]:
    def not_(atom):
        return (~atom).normalize()
    atoms = expr.atoms()
    return frozenset({frozenset({atom, not_(atom)}) for atom in atoms if not_(atom) in atoms})


def _fire(expr: Expr, tokens: frozenset[Place]) -> frozenset[Place]:
    next_tokens = set(tokens)
    for token, sub_expr, lineage in (
            (token, sub_expr, lineage)
            for token in tokens
            for sub_expr in token
            for lineage in filter(lambda x: x[0] == sub_expr, expr.lineaged_subexprs())):
        partial_token = token - {sub_expr}
        if len(lineage) > 1:
            parent = lineage[1]
            pred_and = all(map(lambda x: {x} | partial_token in tokens, parent.exprs))
            pred_or = any(map(lambda x: {x} | partial_token in tokens, parent.exprs))
            if (type(parent) == And and pred_and) or (type(parent) == Or and pred_or):
                next_tokens |= {frozenset({parent} | partial_token)}
    return frozenset(next_tokens)


def _project(expr: Expr, tokens: frozenset[Place]) -> frozenset[Place]:
    projection: set[Place] = set()
    for token in tokens:
        for atom in expr.subexprs():
            projection |= {frozenset(token | {atom})}
    return frozenset(projection)
