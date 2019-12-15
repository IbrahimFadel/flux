#include <iostream>
#include "lexer.h"

using namespace Lexer;

using std::vector;

namespace Parser
{
struct Node
{
  int type;
};
} // namespace Parser

void generate_ast(vector<Token> tokens);