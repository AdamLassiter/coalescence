#include "sequent.h"


// Create a new dynamically-allocated sequent proof
SequentProof *sequent_proof_new() {
    SequentProof *proof = (SequentProof *) malloc(sizeof *proof);
    *proof = (SequentProof) {
        .branches = llist_new(),
        .sequents = llist_new()
    };
    return proof;
}

// Free a dynamically-allocated sequent proof
void sequent_proof_free(SequentProof *proof) {
    for (LListNode *n = proof->sequents->head; n != NULL; n = n->next)
        sequent_proof_free((SequentProof *) n->value);
    free(proof->branches);
    free(proof->sequents);
    free(proof);
}


// Given a proven petri net, backtrack to build a sequent proof
SequentProof *sequent_backtrack(PetriNet *net, size_t *place) {
    bool backtracked = false;
    size_t n = net->places->n;
    size_t *left_tk = (size_t *) calloc(sizeof *left_tk, n);
    size_t *right_tk = (size_t *) calloc(sizeof *right_tk, n);

    // Create a new sequent proof here
    SequentProof *proof = sequent_proof_new();
    
    // For each possible branch (sequent term)
    for (size_t dim = 0; dim < n; dim++) {
        Formula *f = net->symbols[place[dim]];
        Grammar type = f->type;
        // Regardless of how we got here, this is proven
        llist_append(proof->sequents, f);
        
        // Check if one or both paths need to be checked
        size_t arity = type == And ? 2 : type == Or ? 1 : 0;
        bool left_child = false,
             right_child = false;
        
        // FIXME: There exists some bug where we end up with an invalid state and the proof halts
        // This seems to occur on right-branches

        // Where they exist, get both children
        if (arity > 0) { 
            // Left
            memcpy(left_tk, place, sizeof(*place) * n);
            left_tk[dim] = f->left->i;
            petri_net_token_sort(left_tk, n);
            left_child = ndarray_get(net->places, left_tk);
            // Right
            memcpy(right_tk, place, sizeof(*place) * n);
            right_tk[dim] = f->right->i;
            petri_net_token_sort(left_tk, n);
            right_child = ndarray_get(net->places, right_tk);
        }
        
        // Append children as tree branches
        size_t n_children = left_child && right_child ? 2 : left_child || right_child ? 1 : 0;
        // Don't follow in multiple dimensions
        if (n_children >= arity && arity != 0 && !backtracked) {
            backtracked = true;
            switch (arity) {
                case 2:
                    llist_append(proof->branches, sequent_backtrack(net, left_tk));
                    llist_append(proof->branches, sequent_backtrack(net, right_tk));
                break;
                case 1:
                    llist_append(proof->branches, sequent_backtrack(net, left_child ? left_tk : right_tk));
                break;
            }
        }
    }
    free(left_tk);
    free(right_tk);
    return proof;
}


static void sequent_print(LList *sequents) {
    for (LListNode *n = sequents->head; n != NULL; n = n->next) {
        Formula *f = (Formula *) n->value;
        formula_latex(f);
        if (n->next != NULL) printf(", ");
    }
}

void sequent_proof_print(SequentProof *proof) {
    // Count number of children to deduce inference type
    size_t n_children = 0;

    // Print leaves above
    for (LListNode *n = proof->branches->head; n != NULL; n = n->next) {
        SequentProof *q = (SequentProof *) n->value;
        sequent_proof_print(q);
        n_children ++;
    }

    // Print appropriate inference type
    switch (n_children) {
        case 2:
            printf("\\BinaryInfC{$ \\vdash ");
            break;
        case 1:
            printf("\\UnaryInfC{$ \\vdash ");
        break;
        case 0:
            printf("\\AxiomC{$ \\vdash ");
        break;
    }
    sequent_print(proof->sequents);
    printf("$}\n");
}

void sequent_proof_latex(SequentProof *proof) {
    printf("Proof of $ ");
    formula_latex((Formula *) proof->sequents->head->value);
    printf("$\\\\\n");

    printf("\\fbox{\\begin{varwidth}{45\\linewidth}\\begin{prooftree}\n");
    sequent_proof_print(proof);
    printf("\\end{prooftree}\\end{varwidth}\n}\\\\\n");
}

// Recursive sequent printing when a top-substitution is made
static void sequent_substitution_print(Formula *f, char v, bool p) {
    if (p) {
        printf("$ %c := ", v); formula_latex(f); printf("$\\newline\n");
        Formula *t = f->parent;
        f->parent = NULL;
        sequent_recurse(f, true, p);
        f->parent = t;
    }
}


static int sequent_recurse(Formula *formula, bool with_sequent, bool top_optimise) {
    struct timeval start, stop;
    /* 3, 2, 1, Go... */
    gettimeofday(&start, NULL);
    CoalescenceResult r = petri_net_coalescence(formula, with_sequent, top_optimise, with_sequent ? sequent_substitution_print : petri_net_substitution_print);
    gettimeofday(&stop, NULL);
    /* Finish */

    // Print a latex-suitable sequent proof
    if (with_sequent && r.n > 0) {
        // Generate a token for the root node
        size_t *root = (size_t *) calloc(sizeof *root, r.n);
        for (size_t i = 0; i < r.n; i++) {
            root[i] = r.root;
        }

        // Print the proof of the formula root
        SequentProof *p = sequent_backtrack(r.net, root);
        sequent_proof_latex(p);

        free(root);
    }
    
    time_t diff_sec = stop.tv_sec - start.tv_sec,
           diff_usec = stop.tv_usec - start.tv_usec;
    printf(r.n > 0 ? "Solution in %d dimensions.\n" : "No solution found (up to %d dimensions).\n", abs(r.n));
    printf("Time taken: %li sec, %li msec, %li usec\n", diff_sec, diff_usec / 1000, diff_usec % 1000);

    if (with_sequent) printf("\\\\");

    return r.n;
}

#ifdef SEQUENT_PROOF_MAIN
int main(int argc, char *argv[]) {
#else
static int sequent_proof_main(int argc, char *argv[]) {
#endif
    bool top_optimise = false,
         with_sequent = false;
    int c;
    while ((c = getopt(argc, argv, "st")) != -1)
        switch ((char) c) {
            case 's':
                with_sequent = true;
                break;
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
    
    if (with_sequent) printf("\\documentclass[border=1in]{standalone}\n\\usepackage{bpextra,varwidth}\n\\begin{document}\n\\begin{tabular}{@{}l@{}}\n");
    int ret = sequent_recurse(formula, with_sequent, top_optimise);
    if (with_sequent) printf("\\end{tabular}\n\\end{document}\n");
    
    return ret > 0 ? ret : -1;
}
