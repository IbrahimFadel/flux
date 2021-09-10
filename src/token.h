#ifndef TOKEN_H
#define TOKEN_H

typedef struct Position {
  int line, col;
} Position;

typedef enum TokenType {
  TOKTYPE_ILLEGAL,
  TOKTYPE_EOF,

  TOKTYPE_IDENT,
  TOKTYPE_INT,
  TOKTYPE_FLOAT,
  TOKTYPE_STRING_LIT,
  TOKTYPE_CHAR_LIT,

  TOKTYPE_PACKAGE,
  TOKTYPE_PUB,
  TOKTYPE_FN,
  TOKTYPE_RETURN,
  TOKTYPE_MUT,

  TOKTYPE_LPAREN,
  TOKTYPE_RPAREN,
  TOKTYPE_LBRACE,
  TOKTYPE_RBRACE,
  TOKTYPE_SEMICOLON,
  TOKTYPE_ARROW,
  TOKTYPE_COMMA,

  TOKTYPE_MINUS,
  TOKTYPE_ASTERISK,
  TOKTYPE_EQ,

  TOKTYPE_TYPES_BEGIN,

  TOKTYPE_i64,
  TOKTYPE_i32,
  TOKTYPE_i16,
  TOKTYPE_i8,

  TOKTYPE_u64,
  TOKTYPE_u32,
  TOKTYPE_u16,
  TOKTYPE_u8,

  TOKTYPE_TYPES_END,
} TokenType;

typedef struct Token {
  Position pos;
  TokenType type;
  const char *value;
} Token;

TokenType lookup_keyword(const char *str);

#endif