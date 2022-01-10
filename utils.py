#! /usr/bin/env python3

from itertools import chain
from typing import Callable, Iterable, FrozenSet, TypeVar


F = TypeVar('F')
G = TypeVar('G')

def flat_map(func: Callable[[F], Iterable[G]], seq: Iterable[F]) -> Iterable[G]:
    return chain.from_iterable(map(func, seq))


def peek_any(seq: FrozenSet[F]) -> F:
    for elem in seq:
        return elem
    raise Exception('Empty collection')


def loop_into_none(func):
    depth = 0
    memo = set()
    def decorator(*args):
        nonlocal depth
        nonlocal memo
        if args in memo:
            ret = None
        else:
            memo.update({args})
            depth += 1
            ret = func(*args)
            depth -= 1
        if depth == 0:
            memo.clear()
        return ret
    return decorator
