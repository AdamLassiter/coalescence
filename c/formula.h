#pragma once

#include "llist.h"

#include <stdlib.h>
#include <stdio.h>
#include <stdbool.h>


typedef enum Grammar {
    Top = 'T', Bottom = 'F', Atom = 'a', NotAtom = '~', Or = 'v', And = '^',
} Grammar;

typedef enum ExtraGrammar {
    Implies = '>', Implied = '<', Equivalent = '=', NotEquivalent = '#'
} ExtraGrammar;

typedef struct Formula {
    struct Formula *parent, *left, *right;
    Grammar type;
    char symbol;
    size_t i;
} Formula;


Formula *formula_parse(char **);
void formula_free(Formula *);

size_t formula_index(Formula *, size_t);
size_t formula_length(Formula *);
Formula **formula_flatten(Formula *);

void formula_negate(Formula *);
size_t formula_n_free_names(Formula *);

void formula_latex(Formula *);
void formula_print(Formula *);
