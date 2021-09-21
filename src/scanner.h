#ifndef SCANNER_H
#define SCANNER_H

#include <cvec.h>
#include <stdbool.h>

#include "token.h"

const char *read_file(const char *path);

typedef struct Scanner {
  unsigned offset;
  Position pos;
  char ch;
  const char *src;
} Scanner;

char *substr(const char *str, int i, int len);

Scanner *scanner_create(const char *src);
void scanner_destroy(Scanner *s);
void scanner_fatal(Scanner *s, unsigned offset, const char *msg, ...);
void scanner_next(Scanner *s);
char scanner_peek(Scanner *s);
cvector_vector_type(Token *) scan_file(Scanner *s);
Token *scan(Scanner *s);
void scan_whitespace(Scanner *s);
const char *scan_identifier(Scanner *s);
Token *scan_number(Scanner *s);
void scan_digits(Scanner *s, int base, int *invalid_digit_index);
const char *scan_string(Scanner *s);
void scan_escape(Scanner *s, char quote);
const char *scan_char(Scanner *s);

bool is_letter(char c);
bool is_digit(char c);
bool is_hex(char c);

#endif