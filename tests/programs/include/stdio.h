#ifndef RIVET_STDIO_H
#define RIVET_STDIO_H

#include <stddef.h>

typedef struct FILE FILE;

FILE* fopen(const char* filename, const char* mode);
int fclose(FILE* stream);
char* fgets(char* str, int num, FILE* stream);

#endif
