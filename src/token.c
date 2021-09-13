#include "token.h"

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
  if (!strcmp(str, "i64"))
    return TOKTYPE_I64;
  if (!strcmp(str, "i32"))
    return TOKTYPE_I32;
  if (!strcmp(str, "i16"))
    return TOKTYPE_I16;
  if (!strcmp(str, "i8"))
    return TOKTYPE_I8;
  if (!strcmp(str, "u64"))
    return TOKTYPE_u64;
  if (!strcmp(str, "u32"))
    return TOKTYPE_u32;
  if (!strcmp(str, "u16"))
    return TOKTYPE_u16;
  if (!strcmp(str, "u8"))
    return TOKTYPE_u8;
  if (!strcmp(str, "f64"))
    return TOKTYPE_F64;
  if (!strcmp(str, "f32"))
    return TOKTYPE_F32;
  else
    return TOKTYPE_IDENT;
}