#! /usr/bin/env python3

from itertools import chain
from typing import Callable, Generator, Iterable, Collection, TypeVar


F = TypeVar('F')
G = TypeVar('G')


def flat_map(func: Callable[[F], Iterable[G]], seq: Iterable[F]) -> Iterable[G]:
    return chain.from_iterable(map(func, seq))


def unwrap(seq: Collection[F]) -> F:
    assert len(seq) == 1
    return unwrap_any(seq)


def unwrap_any(seq: Iterable[F]) -> F:
    for elem in seq:
        return elem
    raise AssertionError('Empty collection')


def empty_generator() -> Generator[F, None, None]:
    yield from ()
