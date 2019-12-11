#ifndef LEXER_H
#define LEXER_H

#include <iostream>
#include <vector>

using std::string;
using std::vector;

namespace Lexer
{
struct Token
{
  int type;
  string value;
};

enum Types
{
  id = 0,
  kw = 1,
  op = 2,
  lit = 3,
  sep = 4,
  eol = 5
};

} // namespace Lexer

vector<Lexer::Token> generate_tokens(string input);

#endif