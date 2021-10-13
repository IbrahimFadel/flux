#include "token.h"

#include <stdlib.h>
#include <string.h>

TokenType lookup_keyword(const char *str) {
  if (!strcmp(str, "pkg"))
    return TOKTYPE_PACKAGE;
  if (!strcmp(str, "pub"))
    return TOKTYPE_PUB;
  if (!strcmp(str, "fn"))
    return TOKTYPE_FN;
  if (!strcmp(str, "return"))
    return TOKTYPE_RETURN;
  if (!strcmp(str, "mut"))
    return TOKTYPE_MUT;
  if (!strcmp(str, "type"))
    return TOKTYPE_TYPE;
  if (!strcmp(str, "interface"))
    return TOKTYPE_INTERFACE;
  if (!strcmp(str, "struct"))
    return TOKTYPE_STRUCT;
  if (!strcmp(str, "nil"))
    return TOKTYPE_NIL;
  if (!strcmp(str, "if"))
    return TOKTYPE_IF;
  if (!strcmp(str, "else"))
    return TOKTYPE_ELSE;
  if (!strcmp(str, "i64"))
    return TOKTYPE_I64;
  if (!strcmp(str, "i32"))
    return TOKTYPE_I32;
  if (!strcmp(str, "i16"))
    return TOKTYPE_I16;
  if (!strcmp(str, "i8"))
    return TOKTYPE_I8;
  if (!strcmp(str, "u64"))
    return TOKTYPE_U64;
  if (!strcmp(str, "u32"))
    return TOKTYPE_U32;
  if (!strcmp(str, "u16"))
    return TOKTYPE_U16;
  if (!strcmp(str, "u8"))
    return TOKTYPE_U8;
  if (!strcmp(str, "f64"))
    return TOKTYPE_F64;
  if (!strcmp(str, "f32"))
    return TOKTYPE_F32;
  else
    return TOKTYPE_IDENT;
}

void token_destroy(Token *tok) {
  if (tok->type == TOKTYPE_INT || tok->type == TOKTYPE_FLOAT) {
    free((char *)tok->value);
  }
  free(tok);
}