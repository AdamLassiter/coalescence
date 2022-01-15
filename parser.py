#! /usr/bin/env python3

from __future__ import annotations
from typing import Callable, Optional

from expression import Expr


def parse(linput_: str, combinator=lambda x: x, parent='?') -> Expr:
    linput_ = linput_.lstrip()
    lexpr, lindex = _nongreedy_parse(linput_, parent=parent)

    input_ = linput_[lindex + 1:].lstrip()
    expr_fn, index = _operator_parse(input_, lexpr, parent=linput_)

    if expr_fn is None:
        expr = lexpr
    else:
        rinput_ = input_[index + 1:].lstrip()
        rexpr = parse(rinput_, parent=linput_)
        expr = expr_fn(rexpr)

    return combinator(expr)


def _find_closing(input_: str, opening='(', closing=')') -> int:
    next_open = input_.find(opening)
    next_close = input_.find(closing)
    if next_open >= 0 and next_close >= 0 and next_open < next_close:
        matched_close = next_open + 1 + _find_closing(input_[next_open + 1:], opening, closing)
        assert input_[matched_close] == ')'
        next_close = matched_close + 1 + _find_closing(input_[matched_close + 1:], opening, closing)
        assert input_[next_close] == ')'
        return next_close
    elif next_close >= 0:
        assert input_[next_close] == ')'
        return next_close
    else:
        raise Exception('Unmatched open-close in "' + input_ + '"')


def _nongreedy_parse(input_: str, parent='?') -> tuple[Expr, int]:
    if not input_:
        msg = 'Expected expression in %s, got empty' % (parent)
        raise Exception(msg)
    elif input_[0] == '(':
        index = 1 + _find_closing(input_[1:])
        assert input_[index] == ')'
        expr = parse(input_[1:index], parent=input_)
    elif input_[0] == '~':
        expr, index = _nongreedy_parse(input_[1:], input_)
        index += 1
        expr = Expr.Not(expr)
    else:
        index = input_.find(' ')
        atom = input_[0:index] if index >= 0 else input_
        expr = Expr.Atom(atom)
    return (expr, index if index >= 0 else len(input_))


def _operator_parse(input_: str, lexpr: Expr, parent='?') -> tuple[Optional[Callable[[Expr], Expr]], int]:
    if not input_:
        expr_fn = None
    elif input_[0] == '&':
        expr_fn = lambda rexpr: Expr.And(lexpr, rexpr)
    elif input_[0] == '|':
        expr_fn = lambda rexpr: Expr.Or(lexpr, rexpr)
    elif input_[0] == '>':
        expr_fn = lambda rexpr: Expr.Or(Expr.Not(lexpr), rexpr)
    elif input_[0] == '=':
        expr_fn = lambda rexpr: Expr.And(Expr.Or(Expr.Not(lexpr), rexpr), Expr.Or(lexpr, Expr.Not(rexpr)))
    else:
        msg = 'Expected empty, | or & in %s, got %s' % (parent, input_[0])
        raise Exception(msg)
    return (expr_fn, 0)
