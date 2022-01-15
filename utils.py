#! /usr/bin/env python3

from itertools import chain
from typing import Callable, Collection, Iterable, Optional, Tuple, TypeVar


F = TypeVar('F')
G = TypeVar('G')


def flat_map(func: Callable[[F], Iterable[G]], seq: Iterable[F]) -> Iterable[G]:
    return chain.from_iterable(map(func, seq))


def unwrap(seq: Collection[F]) -> F:
    assert len(seq) == 1
    return unwrap_any(seq)


def unwrap_any(seq: Iterable[F]) -> F:
    maybe_elem = maybe_unwrap_any(seq)
    if maybe_elem:
        return maybe_elem
    else:
        raise AssertionError('Empty collection')


def maybe_unwrap_any(seq: Iterable[F]) -> Optional[F]:
    for elem in seq:
        return elem
    else:
        return None


def peek(iter: Iterable[F]) -> Tuple[Optional[F], Iterable[F]]:
    try:
        head = next(iter) # type: ignore
        return head, chain([head], iter)
    except StopIteration:
        return (None, iter)
