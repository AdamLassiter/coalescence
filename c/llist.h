#pragma once

#include <stdbool.h>
#include <stdlib.h>
#include <stdio.h>


typedef struct LListNode {
    struct LListNode *next;
    struct LListNode *prev;
    void *value;
} LListNode;

typedef struct LList {
    LListNode *head;
    LListNode *tail;
    int len;
} LList;


LList *llist_new();
void llist_free(LList *);

LListNode *llist_get_node(LList *, int);
void *llist_get(LList *, int);

void llist_remove_node(LList *, LListNode *);
void llist_remove_head(LList *);
void llist_remove_tail(LList *);
void llist_remove(LList *, int);

void llist_prepend(LList *, void *);
void llist_append_node(LList *, LListNode *);
void llist_append(LList *, void *);
void llist_insert(LList *, void *, int);

bool llist_contains(LList *, void *);
int llist_indexof(LList *, void *);

void llist_print(LList *);
