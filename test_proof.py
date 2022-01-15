
#! /usr/bin/env python3

import unittest

from proof import proof
from parser import parse


class TestBacktrack(unittest.TestCase):
    def test_axiom(self):
        expr = parse('a > a').normalize()
        self.assertTrue(proof(expr) is not None)

    def test_paired_axiom(self):
        expr = parse('(a > a) & (a > a)').normalize()
        self.assertTrue(proof(expr) is not None)

    def test_second_axiom(self):
        expr = parse('(a & b) | (a & ~b) | (~a & b) | (~a & ~b)').normalize()
        self.assertTrue(proof(expr) is not None)

    def test_third_axiom(self):
        expr = parse('(a & b & c) | (a & b & ~c) | (a & ~b & c) | (a & ~b & ~c) | (~a & b & c) | (~a & b & ~c) | (~a & ~b & c) | (~a & ~b & ~c)').normalize()
        self.assertTrue(proof(expr) is not None)


if __name__ == '__main__':
    unittest.main()
