#include "error.h"

#include <stdarg.h>
#include <stdio.h>
#include <stdlib.h>

#define KRED "\033[0;31m"
#define KWHT "\033[0;37m"

void log_error(ERROR_TYPE type, const char *fmt, ...) {
  va_list args;
  va_start(args, fmt);
  switch (type) {
    case ERRTYPE_DRIVER:
      printf("%sdriver error:%s ", KRED, KWHT);
      break;
    case ERRTYPE_LEX:
      printf("%slexer error:%s ", KRED, KWHT);
      break;
    case ERRTYPE_PARSE:
      printf("%sparser error:%s ", KRED, KWHT);
      break;
    case ERRTYPE_TYPECHECK:
      printf("%stypecheck error:%s ", KRED, KWHT);
      break;
    case ERRTYPE_CODEGEN:
      printf("%scodegen error:%s ", KRED, KWHT);
      break;
  }
  vprintf(fmt, args);
  va_end(args);
  exit(1);
}