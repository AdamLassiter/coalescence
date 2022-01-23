#! /usr/bin/env python3

import unittest

from expression import Expr
from parser import parse


class TestParse(unittest.TestCase):
    def test_parse_atom(self):
        self.assertEqual(parse('a'),
                         Expr.Atom('a'))
        self.assertEqual(parse('alpha'),
                         Expr.Atom('alpha'))
        self.assertEqual(parse('~a'),
                         Expr.Not(Expr.Atom('a')))

    def test_parse_operator(self):
        self.assertEqual(parse('a & b'),
                         Expr.And(Expr.Atom('a'), Expr.Atom('b')))
        self.assertEqual(parse('a | b'),
                         Expr.Or(Expr.Atom('a'), Expr.Atom('b')))
        self.assertEqual(parse('~(a & b)'),
                         Expr.Not(Expr.And(Expr.Atom('a'), Expr.Atom('b'))))

    def test_syntactic_sugar(self):
        self.assertEqual(parse('a > b'),
                         Expr.Or(Expr.Not(Expr.Atom('a')), Expr.Atom('b')))
        self.assertEqual(parse('a = b'),
                         Expr.And(Expr.Or(Expr.Not(Expr.Atom('a')), Expr.Atom('b')),
                                  Expr.Or(Expr.Not(Expr.Atom('b')), Expr.Atom('a'))))

    def test_parse_braces(self):
        self.assertEqual(parse('~(a | b)'),
                         Expr.Not(Expr.Or(Expr.Atom('a'), Expr.Atom('b'))))
        self.assertEqual(parse('(a & b) | (c & d)'),
                         Expr.Or(Expr.And(Expr.Atom('a'), Expr.Atom('b')),
                                 Expr.And(Expr.Atom('c'), Expr.Atom('d'))))
        self.assertEqual(parse('(a)'),
                        Expr.Atom('a'))

    def test_parse_braces_nested(self):
        self.assertEqual(parse('(a | (b & c))'),
                        Expr.Or(Expr.Atom('a'), Expr.And(Expr.Atom('b'), Expr.Atom('c'))))
        self.assertEqual(parse('((a))'),
                        Expr.Atom('a'))


if __name__ == '__main__':
    unittest.main()
