#! /usr/bin/env python3

import unittest

from coalescence import coalescence, _spawn
from expression import Expr
from parser import parse


class TestSpawn(unittest.TestCase):
    def test_axiom(self):
        expr = parse('a > a').normalize()
        self.assertEqual(_spawn(expr),
                         {frozenset({Expr.Atom('a'), Expr.NotAtom('a')})})


class TestCoalescence(unittest.TestCase):
    def test_axiom(self):
        expr = parse('a > a').normalize()
        self.assertTrue(coalescence(expr) is not None)

    def test_paired_axiom(self):
        expr = parse('(a > a) & (a > a)').normalize()
        self.assertTrue(coalescence(expr) is not None)

    def test_second_axiom(self):
        expr = parse('(a & b) | (a & ~b) | (~a & b) | (~a & ~b)').normalize()
        self.assertTrue(coalescence(expr) is not None)

    def test_third_axiom(self):
        expr = parse('(a & b & c) | (a & b & ~c) | (a & ~b & c) | (a & ~b & ~c) | (~a & b & c) | (~a & b & ~c) | (~a & ~b & c) | (~a & ~b & ~c)').normalize()
        self.assertTrue(coalescence(expr) is not None)


if __name__ == '__main__':
    unittest.main()
