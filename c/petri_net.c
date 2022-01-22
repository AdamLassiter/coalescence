#include "petri_net.h"


size_t global_fn_dimn;
int lex_arr_cmp(RBTree *tree, RBTreeNode *left, RBTreeNode *right) {
    size_t *left_arr = (size_t *) left->value, *right_arr = (size_t *) right->value;
    for (size_t i = 0; i < global_fn_dimn; i++) {
        if (left_arr[i] < right_arr[i]) {
            return -1;
        } else if (left_arr[i] > right_arr[i]) {
            return 1;
        }
    }
    return 0;
}

static void print_arr(void *arr){
    printf("[");
    for (size_t i = 0; i < global_fn_dimn; i++) {
        printf("%lu", ((size_t *) arr)[i]);
        if (i < global_fn_dimn - 1)
            printf(" ");
    }
    printf("] ");
}

int size_t_cmp_f(const void * a, const void * b) {
    size_t ca = *(size_t *) a, cb = *(size_t *) b;
    return ca > cb ? 1 : ca < cb ? -1 : 0;
}

void petri_net_token_sort(size_t *arr, size_t n) {
    qsort(arr, n, sizeof(*arr), size_t_cmp_f);
}


// Create a new dynamically-allocated petri net
PetriNet *petri_net_new(Formula *formula) {
    PetriNet *net = (PetriNet *) malloc(sizeof *net);
    *net = (PetriNet) {
        .symbols = formula_flatten(formula),
        .len = formula_length(formula),
        .tokens = rbtree_new(lex_arr_cmp),
        .places = ndarray_new(0, NULL, 0)
    };
    return net;
}

// Free a dynamically-allocated petri net
void petri_net_free(PetriNet *net) {
    free(net->symbols);
    rbtree_free(net->tokens, rbtree_node_ptr_dealloc_cb);
    if (net->places != NULL) ndarray_free(net->places);
    free(net);
}


static bool petri_net_remove_redundant(PetriNet *net, size_t *child, size_t *place) {
    size_t n = net->places->n;
    // Delete if the parent has been spawned already
    Formula *this_sym_check, *parent_sym_check;
    bool check = true;
    for (size_t dimn_check = 0; dimn_check < n; dimn_check++) {
        this_sym_check = net->symbols[child[dimn_check]];
        parent_sym_check = this_sym_check->parent;
        check &= (parent_sym_check != NULL);
        if (check) {
            memcpy(place, child, sizeof(*place) * n);
            place[dimn_check] = parent_sym_check->i;
            petri_net_token_sort(place, n);
            check &= ndarray_get(net->places, place);
        }
    }
    
    if (check)
        rbtree_remove(net->tokens, child);
    
    return check;
}

static bool petri_net_fire(PetriNet *net, Formula *from, size_t *from_tk, Formula *to, size_t *to_tk, size_t dimn) {
    size_t n = net->places->n;
    // Remove node from tree but maintain token
    // Perform the transition if the sibling exists or if performing Or
    if ((ndarray_get(net->places, to_tk)) || to->type == Or) {
        rbtree_remove_with_cb(net->tokens, from_tk, rbtree_node_dealloc_cb);
        from_tk[dimn] = to->i;
        petri_net_token_sort(from_tk, n);
        ndarray_set(net->places, from_tk, true);
        rbtree_insert(net->tokens, from_tk);
        return true;
    } else {
        return false;
    }
}

// Perform coalescence algorithm along a single dimension axis
static bool petri_net_1d_coalescence(PetriNet *net, size_t dimn, size_t *place) {
    // size_t *place is a n-large swapspace for storing place coords in
    bool fired = false;
    size_t *token, n = net->places->n;
    Formula *this_sym, *parent_sym;

    // For each token in the net
    RBTreeIter *iter = rbtree_iter_new();
    for (token = (size_t *) rbtree_iter_first(iter, net->tokens); token; token = (size_t *) rbtree_iter_next(iter)) {
        this_sym = net->symbols[token[dimn]];

        if (this_sym->parent == NULL) {
            // Skip if we're at a partial root
            continue;

        } else {
            // Get the parent transition
            parent_sym = this_sym->parent;
            memcpy(place, token, sizeof(*place) * n);
            place[dimn] = parent_sym->i;
            petri_net_token_sort(place, n);
            
            if (ndarray_get(net->places, place)) {
                if ((fired |= petri_net_remove_redundant(net, token, place)))
                    break;
            } else {
                Formula *sibling = (parent_sym->left == this_sym) ? parent_sym->right : parent_sym->left;
                memcpy(place, token, sizeof(*place) * n);
                place[dimn] = sibling->i;
                petri_net_token_sort(place, n);
                if ((fired |= petri_net_fire(net, this_sym, token, parent_sym, place, dimn)))
                    break;
            }
        }
    }
    rbtree_iter_free(iter);

    return fired;
}


// Given a petri net with tokens in n dims, extend to n+1 dims
static RBTree *petri_net_nd_extrapolate(PetriNet *net, size_t n) {
    RBTree *new_tokens = rbtree_new(lex_arr_cmp);
    size_t *old_tk, *new_tk;
    // Extrapolate old tokens 'sideways' into new dimension
    RBTreeIter *iter = rbtree_iter_new();
    for (old_tk = (size_t *) rbtree_iter_first(iter, net->tokens); old_tk; old_tk = (size_t *) rbtree_iter_next(iter)) {
        for (size_t i = 0; i < net->len; i++) {
            new_tk = (size_t *) calloc(sizeof(*new_tk), n);
            memcpy(new_tk, old_tk, sizeof(*new_tk) * (n-1));
            new_tk[n-1] = i;
            petri_net_token_sort(new_tk, n);
            if (rbtree_find(new_tokens, new_tk)) {
                free(new_tk);
            } else {
                rbtree_insert(new_tokens, new_tk);
            }
        }
    }
    rbtree_iter_free(iter);
    return new_tokens;
}

// Given a petri net, create tokens for a 2D case
static RBTree *petri_net_2d_spawn(PetriNet *net) {
    RBTree *new_tokens = rbtree_new(lex_arr_cmp);
    Formula *symbolA, *symbolB;
    size_t *new_tk;
    for (size_t i = 0; i < net->len; i++) {
        symbolA = net->symbols[i];
        for (size_t j = i; j < net->len; j++) {
            symbolB = net->symbols[j];
            if ((symbolA->type == Top || symbolB->type == Top) || // Top rule 
                    (((symbolA->type == Atom && symbolB->type == NotAtom) || // Ax rule
                      (symbolA->type == NotAtom && symbolB->type == Atom)) &&
                     (symbolA->symbol == symbolB->symbol))) {
                new_tk = (size_t *) calloc(sizeof(*new_tk), 2);
                new_tk[0] = i; new_tk[1] = j;
                petri_net_token_sort(new_tk, 2);
                if (rbtree_find(new_tokens, new_tk)) {
                    free(new_tk);
                } else {
                    rbtree_insert(new_tokens, new_tk);
                }
            }
        }
    }
    return new_tokens;
}


// Given a formula f, create a net and try to solve in n dimensions
static PetriNet *petri_net_exhaustive_fire(Formula *f, size_t n) {
    size_t *dims = (size_t *) calloc(sizeof(*dims), n),
           *place = (size_t *) calloc(sizeof(*place), n),
           *root = (size_t *) calloc(sizeof(*root), n);

    PetriNet *net = petri_net_new(f);
    for (size_t i = 0; i < n; i++) {
        dims[i] = net->len;
        root[i] = f->i;
    }
    net->places = ndarray_new(n, dims, sizeof(bool));

    // Create new token tree
    for (size_t i = 2; i <= n; i++) {
        RBTree *new_tokens;
        global_fn_dimn = i;
        if (i == 2)
            new_tokens = petri_net_2d_spawn(net);
        else
            new_tokens = petri_net_nd_extrapolate(net, i);
        rbtree_free(net->tokens, rbtree_node_ptr_dealloc_cb);
        net->tokens = new_tokens;
    }
    // and then populate grid with tokens
    RBTreeIter *iter = rbtree_iter_new();
    for (size_t *tk = (size_t *) rbtree_iter_first(iter, net->tokens); tk; tk = (size_t *) rbtree_iter_next(iter)) {
        // FIXME: There are more tokens than this - one for each permutation
        ndarray_set(net->places, tk, true);
    }
    rbtree_iter_free(iter);

    // Try to fire in all n dimensions, halt if none fired
    bool fired = true;
    while (fired) {
        fired = false;
        for (size_t dimn = 0; dimn < n; dimn++) {
            fired |= petri_net_1d_coalescence(net, dimn, place);
        }
        // Escape early if done
        if (rbtree_find(net->tokens, root))
            break;
    }

    // Clean up and return
    free(dims); free(place); free(root);
    return net;
}

// Given an (exhaustively) fired net, search for proofs of subformulae
SubstitutionResult petri_net_substitute_top(PetriNet *net, Formula *root, char free_var, bool latex_out, PrintSubFn sub_print) {
    Formula *f = (Formula *) malloc(sizeof *f);
    size_t n = net->places->n;
    bool substituted = false;
    
    size_t *place = (size_t *) calloc(sizeof *place, n);
    for (size_t i = 0; i < n; i++)
        place[i] = root->i;
    
    if ((root->type == And) || (root->type == Or)) {
        if (ndarray_get(net->places, place)) {
            sub_print(root, free_var, latex_out);
            
            substituted = true;

            *f = (Formula) {
                .type = Top,
                .symbol = free_var,
                .parent = NULL
            };
            free_var += 1;
        } else {
            SubstitutionResult left = petri_net_substitute_top(net, root->left, free_var, latex_out, sub_print),
                               right = petri_net_substitute_top(net, root->right, left.free_var, latex_out, sub_print);
            free_var = right.free_var;
            substituted |= left.substituted || right.substituted;

            *f = (Formula) {
                .left = left.formula,
                .right = right.formula,
                .type = root->type,
                .symbol = root->symbol,
                .parent = NULL
            };
            f->left->parent = f;
            f->right->parent = f;
        }
    } else {
        *f = (Formula) {
            .type = root->type,
            .symbol = root->symbol,
            .parent = NULL
        };
    }
    
    free(place); 
    formula_index(f, 0);
    return (SubstitutionResult) {
        .formula = f,
        .substituted = substituted,
        .free_var = free_var
    };
}

// Perform coalescence algorithm in all dimensions until halt or out-of-memory error
 CoalescenceResult petri_net_coalescence(Formula *f, bool latex_out, bool top_opt, PrintSubFn sub_print) {
    size_t n_free = formula_n_free_names(f);
    size_t n;
    char substituted = 0;
    char free_var = 'A';
    
    for (n = 2; n <= n_free + 1; (!substituted) ? n++ : substituted--) {
        if (!latex_out) {formula_print(f); printf("\n");}

        // Fire an n-dimensional net exhaustively
        PetriNet *net = petri_net_exhaustive_fire(f, n);
        
        // Check for substitutions, in particular root for Top
        SubstitutionResult res = petri_net_substitute_top(net, f, free_var, !(latex_out || top_opt), sub_print);
        Formula *f_next = res.formula;

        // Check if we have reached the root
        if (f_next->type == Top) {
            formula_free(f_next);
            return (CoalescenceResult) {
                .net = net,
                .n = (int) n,
                .root = (int) f->i
            };
        } else if (res.substituted && latex_out && top_opt) {
            SubstitutionResult print = petri_net_substitute_top(net, f, free_var, latex_out, sub_print);
            free_var = print.free_var;
            substituted = (char) print.substituted;
            formula_free(print.formula);
        }
        
        // Optimise by substituting proofs of subformulae for T
        if (top_opt) {
            formula_free(f);
            f = f_next;
        } else {
            formula_free(f_next);
        }
        petri_net_free(net);
    }

    return (CoalescenceResult) {
        .net = NULL,
        .n = 1 - (int) n,
        .root = -1
    };
}


// Prettyprint a petri net
void petri_net_print(PetriNet *net) {
    printf("~~ Petri Net at %p\n", net);

    for (size_t i = 0; i < net->places->dims[0]; i++) {
        printf("[%lu] ", i);
        formula_print(net->symbols[i]);
        printf("\n");
    }
    printf("\n");

    rbtree_print(net->tokens, print_arr);
    printf("\n");

    ndarray_print(net->places);
    printf("\n~~ End of Petri Net\n");
}

// What to do when a Top substitution might be printed
void petri_net_substitution_print(Formula *f, char v, bool p) {
    if (!p) {
        printf("%c := ", v); formula_print(f); printf("\n");
    }
}


#ifdef PETRI_NET_MAIN
int main(int argc, char *argv[]) {
#else
static int petri_net_main(int argc, char *argv[]) {
#endif
    bool latex_out = false,
         top_optimise = false;
    int c;
    while ((c = getopt(argc, argv, "t")) != -1)
        switch ((char) c) {
            case 't':
                top_optimise = true;
                break;
            case '?':
                if (isprint(optopt))
                  fprintf(stderr, "Unknown option `-%c'.\n", optopt);
                else
                  fprintf(stderr,
                           "Unknown option character `\\x%x'.\n", optopt);
                return 1;
            default:
                abort();
        }
    
    char *string = argv[optind];
    Formula *formula = formula_parse(&string);
    
    struct timeval start, stop;
    /* 3, 2, 1, Go... */
    gettimeofday(&start, NULL);
    CoalescenceResult r = petri_net_coalescence(formula, latex_out, top_optimise, petri_net_substitution_print);
    gettimeofday(&stop, NULL);
    /* Finish */
    
    time_t diff_sec = stop.tv_sec - start.tv_sec,
           diff_usec = stop.tv_usec - start.tv_usec;
    printf(r.n > 0 ? "Solution in %d dimensions.\n" : "No solution found (up to %d dimensions).\n", abs(r.n));
    printf("Time taken: %li sec, %li msec, %li usec\n", diff_sec, diff_usec / 1000, diff_usec % 1000);
    
    return r.n > 0 ? r.n : -1;
}

