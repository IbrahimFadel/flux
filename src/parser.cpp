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
  std::vector<Binary_Operation_Node> bin_op_nodes;
  while (tokens[x]->value != ";" && tokens[x]->type != Token_Types::eol)
  {
    if(tokens[x]->type == Token_Types::op && tokens[x]->value == "+")
    {
      Binary_Operation_Node bin_op_node;
      Expression_Node *left_expr = new Expression_Node();
      Expression_Node *right_expr = new Expression_Node();
      Number_Expression_Node left_number_expr;
      String_Expression_Node left_string_expr;
      Number_Expression_Node right_number_expr;
      String_Expression_Node right_string_expr;

      bool is_number_expression = false;

      for(int j = x - 1; j >= i; j--)
      {
        std::shared_ptr<Token> left = tokens[j];
        if(left->type == Token_Types::op)
        {
          left_number_expr.ops.push_back(left->value);
        }
        else if(is_number(left->value))
        {
          is_number_expression = true;
          left_number_expr.numbers.push_back(left->value);
        }
        else
        {
          is_number_expression = false;
        }
      }

      if(is_number_expression)
      {
        left_expr->type = Expression_Types::NumberExpression;
        left_expr->number_expression = left_number_expr;
      }
      else
      {
        left_expr->type = Expression_Types::StringExpression;
        left_expr->string_expression = left_string_expr;
      }

      int j = x + 1;
      while(tokens[j]->type != Token_Types::eol && tokens[j]->value != ";")
      {
        std::shared_ptr<Token> right = tokens[j];
        if(right->type == Token_Types::op)
        {
          right_number_expr.ops.push_back(right->value);
        }
        else if(is_number(right->value))
        {
          is_number_expression = true;
          right_number_expr.numbers.push_back(right->value);
        }
        else
        {
          is_number_expression = false;
        }
        j++;
      }

      if(is_number_expression)
      {
        right_expr->type = Expression_Types::NumberExpression;
        right_expr->number_expression = right_number_expr;
      }
      else
      {
        right_expr->type = Expression_Types::StringExpression;
        right_expr->string_expression = right_string_expr;
      }


      bin_op_node.left = left_expr;
      bin_op_node.right = right_expr;
      bin_op_node.op = tokens[x]->value;

      bin_op_nodes.push_back(bin_op_node);
    }
    x++;
  }

  for(auto node : bin_op_nodes)
  {
    cout << "BIN OP NODE: " << endl;
    cout << "LEFT: " << endl;

    for(auto num : std::get<Number_Expression_Node>(node.left->number_expression).numbers)
    {
      cout << num << " ";
    }

    cout << endl;

    for(auto op : std::get<Number_Expression_Node>(node.left->number_expression).ops)
    {
      cout << op << " ";
    }

    cout << endl;
  }

  return expr_node;
};