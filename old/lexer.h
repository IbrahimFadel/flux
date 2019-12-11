#ifndef LEXER_H
#define LEXER_H

#include <iostream>

namespace Lexer
{
struct Token
{
  int type;
  std::string value;
};

enum Types
{
  id = 0,
  op = 1,
  num = 2,
  eol = 3,
  kw = 4,
  str = 5,
  opr = 6,
  cpr = 7,
  odq = 8,
  cdq = 9
};
} // namespace Lexer

char *get_file_input(const char *path);
Lexer::Token create_token(int type, std::string value);
std::vector<Lexer::Token> generate_tokens(std::string input);

#endif