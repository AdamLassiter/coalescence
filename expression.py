#! /usr/bin/env python3

from __future__ import annotations
from dataclasses import dataclass
from typing import Generator

from utils import flat_map, unwrap


@dataclass(frozen = True)
class Expr:
    exprs: frozenset[Expr]
    parity: bool = True

    @staticmethod
    def And(*exprs) -> Expr:
        return And(frozenset(exprs))

    @staticmethod
    def Or(*exprs) -> Expr:
        return Or(frozenset(exprs))

    @staticmethod
    def Atom(expr: str) -> Expr:
        return Atom(frozenset([expr])) # type: ignore[list-item]

    @staticmethod
    def NotAtom(expr: str) -> Expr:
        return NotAtom(frozenset([expr]), parity=False) # type: ignore[list-item]

    @staticmethod
    def Not(expr: Expr) -> Expr:
        return Not(frozenset([expr]))

    def __invert__(self) -> Expr:
        return NotImplemented

    def __str__(self):
        return type(self).__name__ + '{' + ', '.join(map(str, self.exprs)) + '}'

    def __repr__(self):
        return self.__str__()

    def normalize(self) -> Expr:
        return type(self)(frozenset(map(lambda x: x.normalize(), self.exprs)))

    def subexprs(self) -> Generator[Expr, None, None]:
        yield from map(lambda x: x[0], self.lineaged_subexprs())

    def lineaged_subexprs(self) -> Generator[list[Expr], None, None]:
        yield [self]
        for expr in self.exprs:
            yield from map(lambda x: x + [self], expr.lineaged_subexprs())

    def atoms(self) -> Generator[Expr, None, None]:
        yield from filter(lambda x: isinstance(x, Atom), self.subexprs())


class And(Expr):
    def __invert__(self) -> Expr:
        return Or(frozenset(map(lambda x: ~x, self.exprs)))

    def normalize(self) -> Expr:
        flatten = lambda x: x.exprs if type(x) == type(self) else [x]
        normal = lambda x: x.normalize()
        norm = frozenset(flat_map(flatten, map(normal, self.exprs)))
        if len(norm) > 1:
            return type(self)(norm)
        else:
            return unwrap(norm)


class Or(And):
    def __invert__(self) -> Expr:
        return And(frozenset(map(lambda x: ~x, self.exprs)))


class Atom(Expr):
    def __invert__(self) -> Expr:
        return NotAtom(self.exprs, parity=False)

    def sub_exprs(self) -> Generator[Expr, None, None]:
        yield self

    def normalize(self) -> Expr:
        return self

    def lineaged_subexprs(self) -> Generator[list[Expr], None, None]:
        yield [self]


class NotAtom(Atom):
    def __invert__(self) -> Expr:
        return Atom(self.exprs)


class Not(Expr):
    def __invert__(self) -> Expr:
        return unwrap(self.exprs)

    def normalize(self) -> Expr:
        return ~(~self).normalize()
