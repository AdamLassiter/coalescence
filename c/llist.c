#include "llist.h"


// Create a new dynamically-allocated linked list
LList *llist_new() {
    LList *list = (LList *) malloc(sizeof(LList));
    *list = (LList) {
        .head = NULL,
        .tail = NULL,
        .len = 0
    };
    return list;
}

// Free a dynamically-allocated linked list
void llist_free(LList *l) {
    while (l->tail != NULL)
        llist_remove_tail(l);
    free(l);
}


// Get the i'th node of a linked list
LListNode *llist_get_node(LList *l, int i) {
    if (l == NULL) return NULL;
    LListNode *m;
    if (0 < i && i < l->len / 2) {
        for (m = l->head; i-- > 0; m = m->next) ;
    } else if (l->len / 2 < i && i < l->len - 1) {
        for (m = l->tail; i-- > 0; m = m->prev) ;
    } else {
        return NULL;
    }
    return m;
}

// Create a singleton llist from a given object
static void llist_from_empty(LList *l, void *o) {
    LListNode *n = (LListNode *) malloc(sizeof(*n));
    *n = (LListNode) {
        .value = o,
        .prev = NULL,
        .next = NULL
    };
    *l = (LList) {
        .head = n,
        .tail = n,
        .len = 1
    };
}

// Get the i'th value of a linked list
void *llist_get(LList *l, int i) {
    return llist_get_node(l, i)->value;
}


// Delete the head value off a llist
void llist_remove_head(LList *l) {
    if (l == NULL) return;
    LListNode *t = l->head;
    l->head = t->next;
    if (l->head != NULL)
        l->head->prev = t->prev;
    free(t);
    l->len--;
}

// Delete the tail off a llist
void llist_remove_tail(LList *l) {
    if (l == NULL) return;
    LListNode *t = l->tail;
    l->tail = t->prev;
    if (l->tail != NULL)
        l->tail->next = t->next;
    free(t);
    l->len--;
}

// Delete a given node off a llist ( O(1) time )
void llist_remove_node(LList *l, LListNode *n) {
    if (l == NULL) return;
    n->prev->next = n->next; n->next->prev = n->prev;
    free(n);
    l->len--;
}

// Delete the n'th node off a llist ( O(n) time )
void llist_remove(LList *l, int i) {
    if (l == NULL) return;
    if (i == 0) {
        llist_remove_head(l);
    } else if (i == l->len - 1) {
        llist_remove_tail(l);
    } else {
        llist_remove_node(l, llist_get_node(l, i));
    }
}


// Append an value to the head of a llist
void llist_prepend(LList *l, void *o) {
    if (l == NULL) return;
    if (l->len == 0) {
        llist_from_empty(l, o);
    } else {
        LListNode *n = (LListNode *) malloc(sizeof(*n));
        *n = (LListNode) {
            .value = o,
            .next = l->head,
            .prev = l->head->prev
        };
        l->head->prev = n; l->head = n; l->len++;
    }

}

// Remove a node from a llist and move it to the tail inplace
void llist_append_node(LList *l, LListNode *n) {
    if (l->len == 1) return;
    if (n->prev != NULL)
        n->prev->next = n->next;
    if (n->next != NULL)
        n->next->prev = n->prev;
    if (l->head == n)
        l->head = n->next;
    l->tail->next = n;
    n->prev = l->tail;
    l->tail = n;
}

// Append an value to the tail of a llist
void llist_append(LList *l, void *o) {
    if (l == NULL) return;
    if (l->len == 0) {
        llist_from_empty(l, o);
    } else {
        LListNode *n = (LListNode *) malloc(sizeof(*n));
        *n = (LListNode) {
            .value = o,
            .prev = l->tail,
            .next = l->tail->next
        };
        l->tail->next = n; l->tail = n; l->len++;
    }
}

// Insert an value at the i'th position of a llist
void llist_insert(LList *l, void *o, int i) {
    if (l == NULL) return;
    if (i == 0) {
        llist_prepend(l, o);
    } else if (i == l->len - 1) {
        llist_append(l, o);
    } else {
        LListNode *n = (LListNode *) malloc(sizeof(*n)),
        *m = llist_get_node(l, i);
        n->value = o; n->prev = m->prev; n->next = m;
        m->prev = n;
        l->len++;
    }
}


// Is an value in a given llist
bool llist_contains(LList *l, void *o) {
    if (l == NULL) return false;
    for (LListNode *n = l->head; n != NULL; n = n->next)
        if (n->value == o)
            return true;
    return false;
}

// Get the index of a given value
int llist_indexof(LList *l, void *o) {
    if (l == NULL) return -1;
    int i = 0;
    for (LListNode *n = l->head; n != NULL; n = n->next)
        if (n->value == o)
            return i;
        else
            i++;
    return -1;
}


// Prettyprint a llist
void llist_print(LList *l) {
    if (l != NULL)
        for (LListNode *n = l->head; n != NULL; n = n->next)
            printf("%d ", *(int *) n->value);
}
