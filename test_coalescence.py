#! /usr/bin/env python3

import unittest

from coalescence import prove
from parser import parse


class TestCoalescence(unittest.TestCase):
    def test_axiom(self):
        expr = parse('a > a').normalize()
        self.assertTrue(prove(expr) is not None)

    def test_paired_axiom(self):
        expr = parse('(a > a) & (a > a)').normalize()
        self.assertTrue(prove(expr) is not None)

    def test_two_axioms(self):
        expr = parse('(a > a) & (a > a)').normalize()
        self.assertTrue(prove(expr) is not None)

    def test_second_axiom(self):
        expr = parse('(a & b) | (a & ~b) | (~a & b) | (~a & ~b)').normalize()
        self.assertTrue(prove(expr) is not None)


if __name__ == '__main__':
    unittest.main()
