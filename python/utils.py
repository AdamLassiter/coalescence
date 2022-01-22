#! /usr/bin/env python3

from itertools import chain
from typing import Callable, Collection, Iterable, TypeVar


F = TypeVar('F')
G = TypeVar('G')


def flat_map(func: Callable[[F], Iterable[G]], seq: Iterable[F]) -> Iterable[G]:
    return chain.from_iterable(map(func, seq))


def unwrap(seq: Collection[F]) -> F:
    assert len(seq) == 1
    for elem in seq:
        return elem
    raise AssertionError('Collection was not a singleton')
