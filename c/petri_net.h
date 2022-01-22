#pragma once

#include "llist.h"
#include "rbtree.h"
#include "ndarray.h"
#include "formula.h"

#include <ctype.h>
#include <stdlib.h>
#include <stdio.h>
#include <sys/time.h>
#include <unistd.h>


typedef struct PetriNet {
    Formula **symbols;
    size_t len;
    RBTree *tokens;
    NdArray *places;
} PetriNet;

typedef struct CoalescenceResult {
    PetriNet *net;
    int n;
    int root;
} CoalescenceResult;

typedef struct SubstitutionResult {
    Formula *formula;
    bool substituted;
    char free_var;
} SubstitutionResult;

typedef void (*PrintSubFn)(Formula *, char, bool);

void petri_net_token_sort(size_t *, size_t);

PetriNet *petri_net_new(Formula *);
void petri_net_free(PetriNet *);

CoalescenceResult petri_net_coalescence(Formula *, bool, bool, PrintSubFn);

void petri_net_print(PetriNet *);
void petri_net_substitution_print(Formula *, char, bool);
