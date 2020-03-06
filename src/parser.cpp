#include "parser.h"

using namespace Lexer;
// using namespace Parser;

using std::cout;
using std::endl;

void Parser::print_nodes(std::vector<Parser::Node> nodes)
{
  for (int i = 0; i < nodes.size(); i++)
  {
    cout << nodes[i] << endl;
  }
}

std::ostream &operator<<(std::ostream &os, const Parser::Node &node)
{
  if(node.type == Parser::Node_Types::var)
  {
    if(node.variable.type == Variables::integer)
    {
      os << "(INT) VAR = " << node.variable.int_value.int_value << endl;
    }
  }
  return os;
}

std::vector<Parser::Node> Parser::parse_tokens(std::vector<Token> tokens)
{
  std::vector<Parser::Node> nodes;
  Token token;
  Parser::Node node;
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
    nodes.push_back(node);

  end:;
  }
  return nodes;
}

Parser::Node Parser::parse_token(std::vector<Token> tokens, int i)
{
  Parser::Node node;
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

Parser::Node Parser::create_int_node(std::vector<Token> tokens, int i)
{
  Parser::Node node;
  node.type = Parser::Node_Types::var;

  Variables::Variable var;
  var.type = Variables::Variable_Types::integer;
  var.name = tokens[i + 1].value;
  var.int_value = Variables::evaluate_expression(tokens, i + 3);

  node.skip = var.int_value.skip + 3;
  node.variable = var;

  return node;
}