#include "parser.h"

void parse_tokens(std::vector<std::shared_ptr<Token>> tokens)
{
  std::vector<std::shared_ptr<Node>> nodes;

  int i = 0;
  for (auto &token : tokens)
  {
    std::shared_ptr<Node> node;
    switch (token->type)
    {
    case Token_Types::kw:
    {
      if (token->value == "const")
      {
        std::shared_ptr<Constant_Declaration_Node> constant_declaration_node = create_constant_declaration_node(tokens, i);
        // node->node = constant_declaration_node;
      }
    }
    break;
    default:
      break;
    }
    i++;
  }
}

/*
 * TODO 
 * Do syntactical error checking
 * 
*/
std::shared_ptr<Constant_Declaration_Node> create_constant_declaration_node(std::vector<std::shared_ptr<Token>> tokens, int i)
{
  std::shared_ptr<Constant_Declaration_Node> node = std::make_shared<Constant_Declaration_Node>();
  node->name = tokens[i + 1]->value;

  if (tokens[i + 3]->value == "int")
  {
    node->type = Variable_Types::IntType;
  }
  else if (tokens[i + 3]->value == "float")
  {
    node->type = Variable_Types::FloatType;
  }
  else if (tokens[i + 3]->value == "string")
  {
    node->type = Variable_Types::StringType;
  }
  else if (tokens[i + 3]->value == "array")
  {
    node->type = Variable_Types::ArrayType;
  }

  std::unique_ptr<Expression_Node> expr = create_expression_node(tokens, i + 5);

  // node.expression;
  return node;
}

std::unique_ptr<Expression_Node> create_expression_node(std::vector<std::shared_ptr<Token>> tokens, int i)
{
  std::unique_ptr<Expression_Node> expr_node = std::make_unique<Expression_Node>();

  int x = i;
  while (tokens[x]->value != ";" && tokens[x]->type != Token_Types::eol)
  {
    cout << tokens[x]->value << endl;
    x++;
  }

  return expr_node;
};