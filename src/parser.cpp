#include "parser.h"

using namespace Lexer;

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
      os << "(INT) " << node.variable.name << " = " << node.variable.int_value.int_value << endl;
    }
  }
  else if(node.type == Parser::Node_Types::fn)
  {
    os << "FN: " << node.fn.name << endl;
    os << "RETURN: " << node.fn.return_type << endl;
    for(int i = 0; i < node.fn.parameters.size(); i++)
    {
      os << "PARAM " << i << ": " << node.fn.parameters[i] << endl;
    }
    for(int i = 0; i < node.fn.then.tokens.size(); i++)
    {
      os << "TOKEN " << i << ": " << node.fn.then.tokens[i].value << endl;
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
    else if(token.value == "fn")
    {
      node = create_fn_node(tokens, i);
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

Parser::Node Parser::create_fn_node(std::vector<Lexer::Token> tokens, int i)
{
  Parser::Node node;
  node.type = Parser::Node_Types::fn;

  Functions::Function fn;
  fn.name = tokens[i + 1].value;
  fn.parameters = Functions::get_fn_parameters(tokens, i + 4);
  int skip;
  if(fn.parameters.size() == 0)
  {
    skip = 0;
  }
  else {
    skip = fn.parameters.size() + fn.parameters.size() - 1;
  }

  fn.return_type = Functions::get_fn_return_type(tokens, i + 3 + skip + 3);
  fn.then = Functions::get_fn_then(tokens, i + 3 + skip + 5);

  for(int x = 0; x < fn.then.tokens.size(); x++)
  {
    Parser::Node child_node = Parser::parse_token(fn.then.tokens, x);
  }

  node.fn = fn;
  node.skip = skip + fn.then.tokens.size() + 1;
  return node;
}