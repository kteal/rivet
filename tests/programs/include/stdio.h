#ifndef RIVET_STDIO_H
#define RIVET_STDIO_H

#include <stddef.h>

typedef struct FILE FILE;

#define EOF (-1)

FILE* fopen(const char* filename, const char* mode);
int fclose(FILE* stream);
char* fgets(char* str, int num, FILE* stream);
int fgetc(FILE* stream);
int putchar(int c);

#endif
