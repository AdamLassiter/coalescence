#pragma once

#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#include <stdbool.h>


typedef unsigned char byte;

typedef struct NdArray {
    size_t n, *dims, size;
    byte *array;
} NdArray;


NdArray *ndarray_new(size_t, size_t [], size_t);
void ndarray_free(NdArray *);

bool ndarray_get(NdArray *, size_t []);
void ndarray_set(NdArray *, size_t [], bool);

void ndarray_print(NdArray *);
