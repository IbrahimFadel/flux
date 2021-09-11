#include "debug.h"

#include <stdio.h>

void *debug_malloc(size_t size, const char *file, const char *function, unsigned line) {
  void *ptr = malloc(size);
  printf("*** Allocated %lu bytes at address %p *** File: %s, Function: %s, Line: %d\n", size, ptr, file, function, line);
  return ptr;
}

void debug_free(void *ptr, const char *file, const char *function, unsigned line) {
  free(ptr);
  printf("*** Free address %p *** File: %s, Function: %s, Line: %d\n", ptr, file, function, line);
}