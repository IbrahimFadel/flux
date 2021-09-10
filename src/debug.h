#ifndef DEBUG_H
#define DEBUG_H

#include <stdlib.h>

#ifdef DEBUG_MEM

#define malloc(size) debug_malloc(size, __FILE__, __FUNCTION__, __LINE__)
#define free(ptr) debug_free(ptr, __FILE__, __FUNCTION__, __LINE__)

#endif

void *debug_malloc(size_t size, const char *file, const char *function, unsigned line);
void debug_free(void *ptr, const char *file, const char *function, unsigned line);

#endif