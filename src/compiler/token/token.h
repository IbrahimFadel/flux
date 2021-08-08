#ifndef TOKEN_H
#define TOKEN_H

#include <map>
#include <string>

#include "position.h"

namespace Token {

enum TokenType {
  ILLEGAL,
  _EOF,
  COMMENT,

  IDENT,
  INT,
  FLOAT,
  CHAR,
  STRING,

  PLUS,
  MINUS,
  ASTERISK,
  SLASH,
  PERCENT,
  AMPERSAND,

  SEMICOLON,
  COMMA,
  LPAREN,
  RPAREN,
  LBRACE,
  RBRACE,
  ARROW,

  keyword_begin,  // used for iterating and stuff

  FN,
  IF,
  FOR,
  TYPE,
  STRUCT,
  RETURN,
  CONST,
  MUT,
  PUB,

  keyword_end,  // used for iterating and stuff

  types_begin,

  I64,
  I32,
  I16,
  I8,
  U64,
  U32,
  U16,
  U8,
  VOID,

  types_end,
};

static std::map<int, std::string> tokens = {
    {TokenType::ILLEGAL, "ILLEGAL"},
    {TokenType::_EOF, "EOF"},
    {TokenType::COMMENT, "COMMENT"},

    {TokenType::IDENT, "IDENT"},
    {TokenType::INT, "ILLEGAL"},
    {TokenType::FLOAT, "ILLEGAL"},
    {TokenType::CHAR, "ILLEGAL"},
    {TokenType::STRING, "ILLEGAL"},

    {TokenType::PLUS, "+"},
    {TokenType::MINUS, "-"},
    {TokenType::ASTERISK, "*"},
    {TokenType::SLASH, "/"},
    {TokenType::PERCENT, "%"},
    {TokenType::AMPERSAND, "&"},

    {TokenType::SEMICOLON, ";"},
    {TokenType::COMMA, ","},
    {TokenType::LPAREN, "("},
    {TokenType::RPAREN, ")"},
    {TokenType::LBRACE, "{"},
    {TokenType::RBRACE, "}"},
    {TokenType::ARROW, "->"},

    {TokenType::FN, "fn"},
    {TokenType::IF, "if"},
    {TokenType::FOR, "for"},
    {TokenType::TYPE, "type"},
    {TokenType::STRUCT, "struct"},
    {TokenType::RETURN, "return"},
    {TokenType::CONST, "const"},
    {TokenType::MUT, "mut"},
    {TokenType::I64, "i64"},
    {TokenType::I32, "i32"},
    {TokenType::I16, "i16"},
    {TokenType::I8, "i8"},
    {TokenType::U64, "u64"},
    {TokenType::U32, "u32"},
    {TokenType::U16, "u16"},
    {TokenType::U8, "u8"},
    {TokenType::VOID, "void"},
};

static std::map<std::string, TokenType> keywords;

struct Token {
  Position pos;
  TokenType type;
  std::string value;
};

void init();
TokenType lookup(std::string ident);

}  // namespace Token

#endif