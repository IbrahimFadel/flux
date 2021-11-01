#ifndef ERROR_H
#define ERROR_H

typedef enum ERROR_TYPE {
  ERRTYPE_DRIVER,
  ERRTYPE_LEX,
  ERRTYPE_PARSE,
  ERRTYPE_TYPECHECK,
  ERRTYPE_CODEGEN,
} ERROR_TYPE;

void log_error(ERROR_TYPE type, const char *fmt, ...);

#endif