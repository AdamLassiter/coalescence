#pragma once

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdlib.h>
#include <stdbool.h>

#ifndef RB_ITER_MAX_HEIGHT
    #define RB_ITER_MAX_HEIGHT 64 // Tallest allowable tree to iterate
#endif


typedef struct RBTreeNode {
    bool red;
    void *value;
    struct RBTreeNode *link[2];
} RBTreeNode;

typedef struct RBTree {
    RBTreeNode *root;
    int (*cmp) (struct RBTree *, struct RBTreeNode *, struct RBTreeNode *);
    size_t size;
} RBTree;

typedef struct RBTreeIter {
    RBTree *tree;
    RBTreeNode *node;
    RBTreeNode *path[RB_ITER_MAX_HEIGHT];
    size_t top;
} RBTreeIter;

typedef int (rbtree_node_cmp_f) (RBTree *, RBTreeNode *, RBTreeNode *);
typedef void (rbtree_node_f) (RBTree *tree, RBTreeNode *node);


extern rbtree_node_cmp_f rbtree_node_cmp_ptr_cb;
extern rbtree_node_f rbtree_node_ptr_dealloc_cb;
extern rbtree_node_f rbtree_node_dealloc_cb;

RBTree *rbtree_new(rbtree_node_cmp_f);
void rbtree_free(RBTree *, rbtree_node_f);

size_t rbtree_size(RBTree *);
void *rbtree_find(RBTree *, void *);

bool rbtree_insert(RBTree *, void *);
bool rbtree_remove_with_cb(RBTree *, void *, rbtree_node_f);
bool rbtree_remove(RBTree *, void *);

void rbtree_print(RBTree *, void (f)(void *));


RBTreeIter *rbtree_iter_new();
void rbtree_iter_free(RBTreeIter *);

void *rbtree_iter_first(RBTreeIter *, RBTree *);
void *rbtree_iter_last(RBTreeIter *, RBTree *);

void *rbtree_iter_next(RBTreeIter *);
void *rbtree_iter_prev (RBTreeIter *);
