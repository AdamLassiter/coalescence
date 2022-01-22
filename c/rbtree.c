#include "rbtree.h"


static RBTreeNode *rbtree_node_alloc() {
    return (RBTreeNode *) malloc(sizeof(RBTreeNode));
}

static RBTreeNode *rbtree_node_init(RBTreeNode *node, void *value) {
    if (node) {
        node->red = 1;
        node->link[0] = node->link[1] = NULL;
        node->value = value;
    }
    return node;
}

static RBTreeNode *rbtree_node_create(void *value) {
    return rbtree_node_init(rbtree_node_alloc(), value);
}

static void rbtree_node_dealloc(RBTreeNode *node) {
    if (node) {
        free(node);
    }
}

static bool rbtree_node_is_red(const RBTreeNode *node) {
    return node ? node->red : false;
}

static RBTreeNode *rbtree_node_rotate(RBTreeNode *node, int dir) {
    RBTreeNode *result = NULL;
    if (node) {
        result = node->link[!dir];
        node->link[!dir] = result->link[dir];
        result->link[dir] = node;
        node->red = 1;
        result->red = 0;
    }
    return result;
}

static RBTreeNode *rbtree_node_rotate2(RBTreeNode *node, int dir) {
    RBTreeNode *result = NULL;
    if (node) {
        node->link[!dir] = rbtree_node_rotate(node->link[!dir], !dir);
        result = rbtree_node_rotate(node, dir);
    }
    return result;
}


int rbtree_node_cmp_ptr_cb(RBTree *tree, RBTreeNode *a, RBTreeNode *b) {
    return (a->value > b->value) - (a->value < b->value);
}

void rbtree_node_ptr_dealloc_cb(RBTree *tree, RBTreeNode *node) {
    if (tree && node) {
        free(node->value);
        rbtree_node_dealloc(node);
    }
}

void rbtree_node_dealloc_cb(RBTree *tree, RBTreeNode *node) {
    if (tree && node) {
        rbtree_node_dealloc(node);
    }
}


static RBTree *rbtree_alloc() {
    return (RBTree *) malloc(sizeof(RBTree));
}

static RBTree *rbtree_init(RBTree *tree, rbtree_node_cmp_f node_cmp_cb) {
    if (tree) {
        tree->root = NULL;
        tree->size = 0;
        tree->cmp = node_cmp_cb ? node_cmp_cb : rbtree_node_cmp_ptr_cb;
    }
    return tree;
}

RBTree *rbtree_new(rbtree_node_cmp_f node_cb) {
    return rbtree_init(rbtree_alloc(), node_cb);
}

void rbtree_free(RBTree *tree, rbtree_node_f node_cb) {
    if (tree) {
        if (node_cb) {
            RBTreeNode *node = tree->root;
            RBTreeNode *save = NULL;
            
            // Rotate away the left links so that
            // we can treat this like the destruction
            // of a linked list
            while (node) {
                if (node->link[0] == NULL) {

                    // No left links, just kill the node and move on
                    save = node->link[1];
                    node_cb(tree, node);
                    node = NULL;
                } else {
                    
                    // Rotate away the left link and check again
                    save = node->link[0];
                    node->link[0] = save->link[1];
                    save->link[1] = node;
                }
                node = save;
            }
        }
        free(tree);
    }
}


size_t rbtree_size(RBTree *tree) {
    size_t result = false;
    if (tree) {
        result = tree->size;
    }
    return result;
}

void *rbtree_find(RBTree *tree, void *value) {
    void *result = NULL;
    if (tree) {
        RBTreeNode node = { .value = value };
        RBTreeNode *it = tree->root;
        int cmp = 0;
        while (it) {
            if ((cmp = tree->cmp(tree, it, &node))) {
                // If the tree supports duplicates, they should be
                // chained to the right subtree for this to work
                it = it->link[cmp < 0];
            } else {
                break;
            }
        }
        result = it ? it->value : NULL;
    }
    return result;
}


static bool rbtree_insert_node(RBTree *tree, RBTreeNode *node) {
    bool result = false;
    if (tree && node) {
        if (tree->root == NULL) {
            tree->root = node;
            result = true;
        } else {
            RBTreeNode head = { 0 }; // False tree root
            RBTreeNode *g, *t;       // Grandparent & parent
            RBTreeNode *p, *q;       // Iterator & parent
            int dir = 0, last = 0;

            // Set up our helpers
            t = &head;
            g = p = NULL;
            q = t->link[1] = tree->root;

            // Search down the tree for a place to insert
            while (1) {
                if (q == NULL) {

                    // Insert node at the first null link.
                    p->link[dir] = q = node;
                } else if (rbtree_node_is_red(q->link[0]) && rbtree_node_is_red(q->link[1])) {
                
                    // Simple red violation: color flip
                    q->red = 1;
                    q->link[0]->red = 0;
                    q->link[1]->red = 0;
                }

                if (rbtree_node_is_red(q) && rbtree_node_is_red(p)) {

                    // Hard red violation: rotations necessary
                    int dir2 = t->link[1] == g;
                    if (q == p->link[last]) {
                        t->link[dir2] = rbtree_node_rotate(g, !last);
                    } else {
                        t->link[dir2] = rbtree_node_rotate2(g, !last);
                    }
                }
          
                // Stop working if we inserted a node. This
                // check also disallows duplicates in the tree
                if (tree->cmp(tree, q, node) == 0) {
                    break;
                }

                last = dir;
                dir = tree->cmp(tree, q, node) < 0;

                // Move the helpers down
                if (g != NULL) {
                    t = g;
                }

                g = p, p = q;
                q = q->link[dir];
            }

            // Update the root (it may be different)
            tree->root = head.link[1];
        }

        // Make the root black for simplified logic
        tree->root->red = 0;
        ++tree->size;
    }
    
    return result;
}

bool rbtree_insert(RBTree *tree, void *value) {
    return rbtree_insert_node(tree, rbtree_node_create(value));
}


bool rbtree_remove_with_cb(RBTree *tree, void *value, rbtree_node_f node_cb) {
    if (tree->root != NULL) {
        RBTreeNode head = {0}; // False tree root
        RBTreeNode node = { .value = value }; // Value wrapper node
        RBTreeNode *q, *p, *g; // Helpers
        RBTreeNode *f = NULL;  // Found item
        int dir = 1;

        // Set up our helpers
        q = &head;
        g = p = NULL;
        q->link[1] = tree->root;
    
        // Search and push a red node down
        // to fix red violations as we go
        while (q->link[dir] != NULL) {
            int last = dir;

            // Move the helpers down
            g = p, p = q;
            q = q->link[dir];
            dir = tree->cmp(tree, q, &node) < 0;
      
            // Save the node with matching value and keep
            // going; we'll do removal tasks at the end
            if (tree->cmp(tree, q, &node) == 0) {
                f = q;
            }

            // Push the red node down with rotations and color flips
            if (!rbtree_node_is_red(q) && !rbtree_node_is_red(q->link[dir])) {
                if (rbtree_node_is_red(q->link[!dir])) {
                    p = p->link[last] = rbtree_node_rotate(q, dir);
                } else if (!rbtree_node_is_red(q->link[!dir])) {
                    RBTreeNode *s = p->link[!last];
                    if (s) {
                        if (!rbtree_node_is_red(s->link[!last]) && !rbtree_node_is_red(s->link[last])) {

                            // Color flip
                            p->red = 0;
                            s->red = 1;
                            q->red = 1;
                        } else {
                            int dir2 = g->link[1] == p;
                            if (rbtree_node_is_red(s->link[last])) {
                                g->link[dir2] = rbtree_node_rotate2(p, last);
                            } else if (rbtree_node_is_red(s->link[!last])) {
                                g->link[dir2] = rbtree_node_rotate(p, last);
                            }
                            
                            // Ensure correct coloring
                            q->red = g->link[dir2]->red = 1;
                            g->link[dir2]->link[0]->red = 0;
                            g->link[dir2]->link[1]->red = 0;
                        }
                    }
                }
            }
        }

        // Replace and remove the saved node
        if (f) {
            void *tmp = f->value;
            f->value = q->value;
            q->value = tmp;
            
            p->link[p->link[1] == q] = q->link[q->link[0] == NULL];
            
            if (node_cb) {
                node_cb(tree, q);
            }
            q = NULL;
        }

        // Update the root (it may be different)
        tree->root = head.link[1];

        // Make the root black for simplified logic
        if (tree->root != NULL) {
            tree->root->red = 0;
        }

        --tree->size;
    }
    return true;
}

bool rbtree_remove(RBTree *tree, void *value) {
    bool result = false;
    if (tree) {
        result = rbtree_remove_with_cb(tree, value, rbtree_node_ptr_dealloc_cb);
    }
    return result;
}


void rbtree_print(RBTree *tree, void (*print)(void *)) {
    RBTreeIter *iter = rbtree_iter_new();
    for (void *elem = rbtree_iter_first(iter, tree); elem; elem = rbtree_iter_next(iter)) {
        print(elem);
    }
    rbtree_iter_free(iter);
}



static RBTreeIter *rbtree_iter_alloc() {
    return (RBTreeIter *) malloc(sizeof(RBTreeIter));
}

static RBTreeIter *rbtree_iter_init(RBTreeIter *self) {
    if (self) {
        self->tree = NULL;
        self->node = NULL;
        self->top = 0;
    }
    return self;
}

RBTreeIter *rbtree_iter_new() {
    return rbtree_iter_init(rbtree_iter_alloc());
}

void rbtree_iter_free(RBTreeIter *self) {
    if (self) {
        free(self);
    }
}


// Internal function, init traversal object, dir determines whether
// to begin traversal at the smallest or largest valued node.
static void *rbtree_iter_start(RBTreeIter *self, RBTree *tree, int dir) {
    void *result = NULL;
    if (self) {
        self->tree = tree;
        self->node = tree->root;
        self->top = 0;

        // Save the path for later traversal
        if (self->node != NULL) {
            while (self->node->link[dir] != NULL) {
                self->path[self->top++] = self->node;
                self->node = self->node->link[dir];
            }
        }

        result = self->node == NULL ? NULL : self->node->value;
    }
    return result;
}

// Traverse a red black tree in the user-specified direction (0 asc, 1 desc)
static void *rbtree_iter_move(RBTreeIter *self, int dir) {
    // Valgrind hates this... how to iterate and allow deletion
    if (self->node->link[dir] != NULL) {
        // Continue down this branch
        self->path[self->top++] = self->node;
        self->node = self->node->link[dir];
        while ( self->node->link[!dir] != NULL ) {
            self->path[self->top++] = self->node;
            self->node = self->node->link[!dir];
        }

    } else {
        // Move to the next branch
        RBTreeNode *last = NULL;
        do {
            if (self->top == 0) {
                self->node = NULL;
                break;
            }
            last = self->node;
            self->node = self->path[--self->top];
        } while (last == self->node->link[dir]);
    }
    return self->node == NULL ? NULL : self->node->value;
}

void *rbtree_iter_first(RBTreeIter *self, RBTree *tree) {
    return rbtree_iter_start(self, tree, 0);
}

void *rbtree_iter_last(RBTreeIter *self, RBTree *tree) {
    return rbtree_iter_start(self, tree, 1);
}

void *rbtree_iter_next(RBTreeIter *self) {
    return rbtree_iter_move(self, 1);
}

void *rbtree_iter_prev(RBTreeIter *self) {
    return rbtree_iter_move(self, 0);
}

