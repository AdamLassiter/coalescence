#! /usr/bin/env python3

import unittest

from expression import Expr
from parser import parse


class TestNormalization(unittest.TestCase):
    def test_fold_operator(self):
        self.assertEqual(parse('a & b & c').normalize(),
                         Expr.And(Expr.Atom('a'), Expr.Atom('b'), Expr.Atom('c')))
        self.assertEqual(parse('a | b | c').normalize(),
                         Expr.Or(Expr.Atom('a'), Expr.Atom('b'), Expr.Atom('c')))

    def test_inverse_atom(self):
        self.assertEqual(parse('~a').normalize(),
                         Expr.NotAtom('a'))
        self.assertEqual(parse('~~a').normalize(),
                         Expr.Atom('a'))

    def test_inverse_operator(self):
        self.assertEqual(parse('~(a & b)').normalize(),
                         Expr.Or(Expr.NotAtom('a'), Expr.NotAtom('b')))


if __name__ == '__main__':
    unittest.main()
