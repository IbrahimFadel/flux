#include "parser.h"

using namespace Lexer;
using namespace Parser;

using std::cout;
using std::endl;

void Parser::parse_tokens(std::vector<Token> tokens)
{
  Token token;
  Node node;
  int skip = 0;
  int skipped = 0;
  for (int i = 0; i < tokens.size(); i++)
  {
    for (int j = 0; j < skip; j++)
    {
      if (skipped + 1 == skip)
      {
        skip = 0;
        skipped = 0;
        goto end;
      }
      skipped++;
      goto end;
    }
    token = tokens[i];
    node = parse_token(tokens, i);
    skip = node.skip;

  end:;
  }
}

Node Parser::parse_token(std::vector<Token> tokens, int i)
{
  Node node;
  Token token = tokens[i];
  if (token.type == Token_Types::kw)
  {
    if (token.value == "int")
    {
      node = create_int_node(tokens, i);
    }
  }
  return node;
}

Node Parser::create_int_node(std::vector<Token> tokens, int i)
{
  Node node;
  node.type = Node_Types::var;

  Variables::Variable var;
  var.type = Variables::integer;
  var.name = tokens[i + 1].value;
  var.intValue = std::stoi(tokens[i + 3].value);

  node.variable = var;
  node.skip = 4;

  return node;
}