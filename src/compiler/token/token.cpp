#include "token.h"

#include <iostream>

void Token::init() {
  for (int i = keyword_begin + 1; i < keyword_end; i++) {
    keywords.insert(std::pair(tokens[i], (TokenType)i));
  }
  for (int i = types_begin + 1; i < types_end; i++) {
    keywords.insert(std::pair(tokens[i], (TokenType)i));
  }
}

Token::TokenType Token::lookup(std::string ident) {
  if (keywords.count(ident)) {
    return keywords[ident];
  }
  return IDENT;
}