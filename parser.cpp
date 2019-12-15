#include <iostream>
#include "lexer.h"
#include "parser.h"

using namespace Lexer;
using namespace Parser;

using std::cout;
using std::endl;
using std::string;
using std::vector;

void check_token(vector<Token> tokens, int i)
{
  if (tokens[i].type == Types::kw)
  {
  }
}

void generate_ast(vector<Token> tokens)
{
  for (int i = 0; i < tokens.size(); i++)
  {
    Token token = tokens[i];

    check_token(tokens, i);
  }
}