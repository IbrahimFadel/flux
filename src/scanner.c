#include "scanner.h"

#include <stdarg.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

const char *read_file(const char *path) {
  char *buffer = 0;
  long length = 0;
  FILE *f = fopen(path, "rb");

  if (f == NULL) {
    return NULL;
  }

  if (f) {
    fseek(f, 0, SEEK_END);
    length = ftell(f);
    fseek(f, 0, SEEK_SET);
    buffer = malloc(length + 1);
    if (buffer) {
      fread(buffer, 1, length, f);
    }
    fclose(f);
  }
  buffer[length] = '\0';
  return buffer;
}

bool is_letter(char c) {
  char lowerCase = ('a' - 'A') | c;
  return ('a' <= lowerCase && lowerCase <= 'z') || c == '_';
}

bool is_digit(char c) {
  return '0' <= c && c <= '9';
}

bool is_hex(char c) {
  char lowerCase = ('a' - 'A') | c;
  return ('0' <= c && c <= '9') || ('a' <= lowerCase && lowerCase <= 'f');
}

char *substr(const char *str, int i, int len) {
  char *sub = malloc(len + 1);
  strncpy(sub, str + i, len);
  sub[len] = '\0';
  return sub;
}

Scanner *scanner_create(const char *src) {
  Scanner *s = malloc(sizeof(Scanner));
  if (s != NULL) {
    s->src = src;
    s->offset = 0;
    Position starting_pos = {.line = 1, .col = 1};
    s->pos = starting_pos;
    s->ch = s->src[s->offset];
  }
  return s;
}

Token *scan_file(Scanner *s) {
  unsigned num_tokens_allocated = TOKENS_ALLOCATION_FACTOR;
  unsigned tokens_len = 0;
  Token *tokens = malloc(sizeof(Token) * num_tokens_allocated);

  while (s->ch != -1 && s->offset < strlen(s->src)) {
    Token *tok = scan(s);
    if (tokens_len == num_tokens_allocated) {
      num_tokens_allocated += TOKENS_ALLOCATION_FACTOR;
      tokens = realloc(tokens, (sizeof *tokens) * num_tokens_allocated);
    }
    tokens[tokens_len] = *tok;
    tokens_len++;
  }

  Token tok = {.pos = {.col = -1, .line = -1}, .type = TOKTYPE_EOF, .value = "EOF"};
  if (tokens_len == num_tokens_allocated) {
    num_tokens_allocated += 1;
    tokens = realloc(tokens, (sizeof *tokens) * num_tokens_allocated);
  }
  tokens[tokens_len] = tok;
  tokens_len++;

  return tokens;
}

void scanner_fatal(Scanner *s, unsigned offset, const char *msg, ...) {
  va_list args;
  va_start(args, msg);

  printf("%s\n", msg);

  int max_padding = 10;
  unsigned start_pos = offset;
  int len = max_padding;

  if (offset - max_padding >= 0) {
    start_pos -= max_padding;
    len += max_padding + 1;
  } else {
    start_pos = 0;
    len += (offset) + 1;
  }
  if (offset + max_padding >= strlen(s->src)) {
    len -= strlen(s->src) - offset;
  }

  char *str = substr(s->src, start_pos, len);
  printf("%d:%d\t%s\n", s->pos.line, s->pos.col, str);

  exit(1);
}

void scanner_next(Scanner *s) {
  s->offset++;
  s->pos.col++;
  if (s->offset == strlen(s->src)) {
    s->ch = -1;
  } else {
    s->ch = s->src[s->offset];
  }
}

char scanner_peek(Scanner *s) {
  if (s->offset + 1 < strlen(s->src)) {
    return s->src[s->offset + 1];
  }
  return -1;
}

const char *scan_identifier(Scanner *s) {
  int initial_offset = s->offset;
  scanner_next(s);
  while (is_letter(s->ch) || is_digit(s->ch)) {
    scanner_next(s);
  }
  int len = s->offset - initial_offset;
  char *str = substr(s->src, initial_offset, len);
  return str;
}

void scan_digits(Scanner *s, int base, int *invalid_digit_index) {
  if (base <= 10) {
    char max = '0' + base;  // maximum number allowed for this number system
    while (is_digit(s->ch) || s->ch == '_') {
      if (s->ch >= max && *invalid_digit_index < 0 && s->ch != '_') {
        *invalid_digit_index = s->offset;
      }

      scanner_next(s);
    }
  } else {
    while (is_hex(s->ch) || s->ch == '_') {
      scanner_next(s);
    }
  }
}

Token *scan_number(Scanner *s) {
  int initial_offset = s->offset;
  Token *tok = malloc(sizeof(Token));

  int base = 10;
  int invalid_digit_index = -1;

  if (s->ch != '.') {
    tok->type = TOKTYPE_INT;
    if (s->ch == '0') {
      scanner_next(s);
      switch (s->ch) {
        case 'x':
          scanner_next(s);
          base = 16;
          break;
        case 'b':
          scanner_next(s);
          base = 2;
        default:
          break;
      }
    }

    scan_digits(s, base, &invalid_digit_index);
  }

  if (s->ch == '.') {
    tok->type = TOKTYPE_FLOAT;
    if (base != 10) {
      scanner_fatal(s, s->offset, "floating point numbers are only permitted in base 10");
    }
    scanner_next(s);
    scan_digits(s, base, &invalid_digit_index);
  }

  int len = s->offset - initial_offset;
  char *val = malloc(len + 1);
  strncpy(val, s->src + initial_offset, len);
  val[len] = '\0';
  tok->value = val;
  if (invalid_digit_index >= 0) {
    const char *lit_name = "";
    switch (base) {
      case 2:
        lit_name = "binary number";
        break;
      case 10:
        lit_name = "decimal number";
        break;
      case 16:
        lit_name = "hexidecimal number";
        break;
      default:
        lit_name = "decimal number";
        break;
    }
    scanner_fatal(s, invalid_digit_index, "invalid digit %s\n", lit_name);
  }

  return tok;
}

void scan_escape(Scanner *s, char quote) {
  int initial_offset = s->offset;

  if (s->ch == quote)
    return scanner_next(s);
  switch (s->ch) {
    case 'r':
    case 't':
    case 'n':
      return scanner_next(s);
    default:
      return scanner_fatal(s, initial_offset, "unknown escape sequence");
  }
}

const char *scan_string(Scanner *s) {
  int initial_offset = s->offset;

  while (true) {
    if (s->ch == '\n' || s->ch < 0) {
      scanner_fatal(s, initial_offset, "string literal not terminated (are you missing a \"?)");
      break;
    }
    if (s->ch == '"') {
      break;
    }
    if (s->ch == '\\') {
      scan_escape(s, '"');
    }
    scanner_next(s);
  }

  const char *str = substr(s->src, initial_offset, s->offset - initial_offset);
  scanner_next(s);
  return str;
}

const char *scan_char(Scanner *s) {
  int initial_offset = s->offset;
  int n = 0;

  while (true) {
    if (s->ch == '\n' || s->ch < 0) {
      scanner_fatal(s, initial_offset, "char literal not terminated (you might be missing a closing ')");
      break;
    }
    if (s->ch == '\'') {
      break;
    }
    if (s->ch == '\\') {
      scan_escape(s, '\'');
    }
    scanner_next(s);
    n++;
  }

  if (n != 1) {
    scanner_fatal(s, initial_offset, "invalid char literal");
  }

  const char *str = substr(s->src, initial_offset, s->offset - initial_offset);
  scanner_next(s);
  return str;
}

Token *scan(Scanner *s) {
  scan_whitespace(s);

  Token *tok = malloc(sizeof(Token));
  tok->type = TOKTYPE_ILLEGAL;
  tok->value = "";

  char ch = s->ch;
  if (is_letter(ch)) {
    tok->value = scan_identifier(s);
    if (strlen(tok->value) > 1) {
      tok->type = lookup_keyword(tok->value);
    } else {
      tok->type = TOKTYPE_IDENT;
    }
  } else if (is_digit(ch) || (ch == '.' && is_digit(scanner_peek(s)))) {
    tok = scan_number(s);
  } else {
    scanner_next(s);
    switch (ch) {
      case ' ':
        scanner_next(s);
        break;
      case '\n':
        s->pos.line++;
        s->pos.col = 0;
        scanner_next(s);
        break;
      case ';':
        tok->type = TOKTYPE_SEMICOLON;
        tok->value = ";";
        break;
      case '"':
        tok->type = TOKTYPE_STRING_LIT;
        tok->value = scan_string(s);
        break;
      case '\'':
        tok->type = TOKTYPE_CHAR_LIT;
        tok->value = scan_char(s);
        break;
      case '(':
        tok->type = TOKTYPE_LPAREN;
        tok->value = "(";
        break;
      case ')':
        tok->type = TOKTYPE_RPAREN;
        tok->value = ")";
        break;
      case '{':
        tok->type = TOKTYPE_LBRACE;
        tok->value = "{";
        break;
      case '}':
        tok->type = TOKTYPE_RBRACE;
        tok->value = "}";
        break;
      case '*':
        tok->type = TOKTYPE_ASTERISK;
        tok->value = "*";
        break;
      case ',':
        tok->type = TOKTYPE_COMMA;
        tok->value = ",";
        break;
      case '=':
        tok->type = TOKTYPE_EQ;
        tok->value = "=";
        break;
      case '-':
        tok->type = TOKTYPE_MINUS;
        tok->value = "-";
        if (s->ch == '>') {
          tok->type = TOKTYPE_ARROW;
          tok->value = "->";
          scanner_next(s);
        }
        break;
    }
  }

  scan_whitespace(s);

  return tok;
}

void scan_whitespace(Scanner *s) {
  while (s->ch == ' ' || s->ch == '\t' || s->ch == '\n' || s->ch == '\r') {
    if (s->ch == '\n') {
      s->pos.line++;
      s->pos.col = 0;
    }
    scanner_next(s);
  }
}