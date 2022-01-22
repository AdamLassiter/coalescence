#include "ndarray.h"


// Create a new dynamically-allocated n-dim array
NdArray *ndarray_new(size_t n, size_t dims[], size_t s) {
    size_t size = 1;
    for (size_t i = 0; i < n; i++) {
        size *= dims[i];
    }
    size_t d_size = sizeof(*dims) * n;

    NdArray *array = (NdArray *) malloc(sizeof(*array));
    *array = (NdArray) {
        .array = (byte *) calloc(size * s / 8 + 1, 1),
        .n = n,
        .dims = (size_t *) malloc(d_size),
        .size = s
    };
    if (array->array == NULL) {
        fprintf(stderr, "Error allocating array: out of memory\n");
        exit(1);
    }
    memcpy(array->dims, dims, d_size);

    return array;
}

// Free a dynamically-allocated array
void ndarray_free(NdArray *array){
    free(array->dims);
    free(array->array);
    free(array);
}


// Return a pointer into the array at a given index
static void *ndarray_elem(NdArray *array, size_t index[]) {
    if (array->n == 0) {
        return array->array;
    }

    int scalar_index = index[0];
    for (size_t n = 1; n < array->n; n++) {
        scalar_index = scalar_index * array->dims[n] + index[n];
    }

    return array->array + scalar_index * array->size / 8;
}

byte MASKS[8] = {0x01, 0x02, 0x04, 0x08, 0x10, 0x20, 0x40, 0x80};
void ndarray_set(NdArray *array, size_t index[], bool val) {
    byte offset = index[array->n - 1] % 8;
    if (val)
        *(byte *) ndarray_elem(array, index) |= MASKS[offset];
    else
        *(byte *) ndarray_elem(array, index) &= ~MASKS[offset];
}

bool ndarray_get(NdArray *array, size_t index[]) {
    byte offset = index[array->n - 1] % 8;
    byte val = *(byte *) ndarray_elem(array, index);
    return (val & MASKS[offset]) >> offset;
}


// Prettyprint in n dimensions... oh boy
void ndarray_print(NdArray *array) {
    // dude, pls
    printf("%d-dimensional array [", (int) array->n);
    for (size_t i = 0; i < array->n; i++) {
        printf("%d", (int) array->dims[i]);
        if (i < array->n - 1) printf(" ");
    }
    printf("]");
    if (array->n == 2) {
        printf("\n");
        for (size_t i = 0; i < array->dims[0]; i++) {
            for (size_t j = 0; j < array->dims[1]; j++) {
                size_t index[] = {i, j};
                if (i <= j)
                    printf("%d ", *(char *) ndarray_elem(array, index));
                else
                    printf("  ");
            }
            if (i < array->dims[0] - 1)
                printf("\n");
        }
    }
}


#ifdef NDARRAY_MAIN
int main(int argc, char *argv[]) {
#else
static int ndarray_main(int argc, char *argv[]) {
#endif
    return 0;
}
