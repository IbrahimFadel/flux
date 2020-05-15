#include "parser.h"

std::vector<Node *> parse_tokens(std::vector<std::shared_ptr<Token>> tokens)
{
  std::vector<Node *> nodes;

  for (int i = 0; i < tokens.size(); i++)
  {
    Node *node = parse_token(tokens, i);
    nodes.push_back(node);

    if (node->skip > 0)
    {
      i += node->skip;
      continue;
    }
  }

  return nodes;
}

Node *parse_token(std::vector<std::shared_ptr<Token>> tokens, int i)
{
  Node *node = new Node();
  switch (tokens[i]->type)
  {
  case Token_Types::kw:
  {
    if (tokens[i]->value == "const")
    {
      Constant_Declaration_Node *constant_declaration_node = create_constant_declaration_node(tokens, i);
      node->type = Node_Types::ConstantDeclarationNode;
      node->constant_declaration_node = constant_declaration_node;
    }
    else if (tokens[i]->value == "fn")
    {
      Function_Declaration_Node *function_declaration_node = create_function_declaration_node(tokens, i);
      node->type = Node_Types::FunctionDeclarationNode;
      node->function_declaration_node = function_declaration_node;
      node->skip = function_declaration_node->skip;
    }
    else if (tokens[i]->value == "return")
    {
      Return_Node *return_node = create_return_node(tokens, i);
      node->type = Node_Types::ReturnNode;
      node->return_node = return_node;
    }
    break;
  }
  default:
    break;
  }

  return node;
}

Return_Node *create_return_node(std::vector<std::shared_ptr<Token>> tokens, int i)
{
  Return_Node *node = new Return_Node();

  std::unique_ptr<Expression_Node> return_expr = create_expression_node(tokens, i + 1);
  node->return_expression = std::move(return_expr);

  return node;
}

Function_Declaration_Node *create_function_declaration_node(std::vector<std::shared_ptr<Token>> tokens, int i)
{
  Function_Declaration_Node *node = new Function_Declaration_Node();

  node->name = tokens[i + 1]->value;

  //! This doesn't work for arrays. Let's implement that later
  std::string return_type = tokens[i + 3]->value;
  if (return_type == "float")
  {
    node->return_type = Variable_Types::FloatType;
  }
  else if (return_type == "int")
  {
    node->return_type = Variable_Types::IntType;
  }
  else
  {
    std::cerr << "Function return type: " << return_type << " not supported yet" << endl;
    return nullptr;
  }

  auto parameters = get_function_declaration_parameters(tokens, i + 6);
  node->parameters = parameters;

  int then_start = i + 5;
  if (parameters.size() == 0)
  {
    then_start += 4;
  }
  else
  {
    then_start += (parameters.size() * 3) + parameters.size() - 1 + 4;
  }

  Then then = get_function_declaration_then(tokens, then_start);
  for (auto &node : then.nodes)
  {
    node->parent = node;
  }
  node->then = then;

  node->skip = node->then.tokens.size();

  return node;
}

Then get_function_declaration_then(std::vector<std::shared_ptr<Token>> tokens, int i)
{
  Then then;

  for (int x = i; x < tokens.size(); x++)
  {
    if (tokens[x]->value == "}")
      break;

    Node *node = parse_token(tokens, x);
    then.nodes.push_back(node);
    then.tokens.push_back(tokens[x]);
  }

  return then;
}

std::map<std::string, Variable_Types> get_function_declaration_parameters(std::vector<std::shared_ptr<Token>> tokens, int i)
{
  std::map<std::string, Variable_Types> parameters;

  int comma_interval = 0;
  int name_type_interval = 0;
  for (int x = i; x < tokens.size(); x++)
  {
    if (tokens[x]->value == ")")
      break;

    if (comma_interval % 2 == 0)
    {
      if (name_type_interval % 2 == 0)
      {
        Variable_Types type = get_variable_type_from_string(tokens[x + 2]->value);
        parameters[tokens[x]->value] = type;
      }
      name_type_interval++;
    }

    comma_interval++;
  }

  return parameters;
}

Variable_Types get_variable_type_from_string(std::string type)
{
  if (type == "int")
  {
    return Variable_Types::IntType;
  }
  else if (type == "float")
  {
    return Variable_Types::FloatType;
  }
}

Constant_Declaration_Node *create_constant_declaration_node(std::vector<std::shared_ptr<Token>> tokens, int i)
{
  Constant_Declaration_Node *node = new Constant_Declaration_Node();
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
  node->expression = std::move(expr);

  // node.expression;
  return node;
}

std::unique_ptr<Expression_Node> create_expression_node(std::vector<std::shared_ptr<Token>> tokens, int i)
{
  std::unique_ptr<Expression_Node> expr_node = std::make_unique<Expression_Node>();
  // expr_node->type = Expression_Types::NumberExpression;

  int x = i;
  std::vector<Binary_Operation_Node> bin_op_nodes;
  while (tokens[x]->value != ";" && tokens[x]->type != Token_Types::eol)
  {
    if (tokens[x]->type == Token_Types::op && tokens[x]->value == "+")
    {
      Binary_Operation_Node bin_op_node;
      Expression_Node *left_expr = new Expression_Node();
      Expression_Node *right_expr = new Expression_Node();
      Number_Expression_Node left_number_expr;
      String_Expression_Node left_string_expr;
      Number_Expression_Node right_number_expr;
      String_Expression_Node right_string_expr;

      bool is_number_expression = false;

      for (int j = x - 1; j >= i; j--)
      {
        std::shared_ptr<Token> left = tokens[j];
        if (left->type == Token_Types::op)
        {
          left_number_expr.ops.push_back(left->value);
        }
        else if (is_number(left->value))
        {
          is_number_expression = true;
          left_number_expr.numbers.push_back(left->value);
        }
        else
        {
          is_number_expression = false;
        }
      }

      if (is_number_expression)
      {
        left_expr->type = Expression_Types::NumberExpression;
        expr_node->type = Expression_Types::NumberExpression;
        left_expr->number_expression = left_number_expr;
      }
      else
      {
        left_expr->type = Expression_Types::StringExpression;
        left_expr->string_expression = left_string_expr;
      }

      int j = x + 1;
      while (tokens[j]->type != Token_Types::eol && tokens[j]->value != ";")
      {
        std::shared_ptr<Token> right = tokens[j];
        if (right->type == Token_Types::op)
        {
          right_number_expr.ops.push_back(right->value);
        }
        else if (is_number(right->value))
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

      if (is_number_expression)
      {
        right_expr->type = Expression_Types::NumberExpression;
        expr_node->type = Expression_Types::NumberExpression;
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

  return expr_node;
};

void print_nodes(std::vector<Node *> nodes)
{
  for (auto &node : nodes)
  {
    switch (node->type)
    {
    case Node_Types::ConstantDeclarationNode:
    {
      Constant_Declaration_Node *constant_declaration_node = std::get<Constant_Declaration_Node *>(node->constant_declaration_node);
      cout << "Constant Declaration Node: " << constant_declaration_node->name << endl;
      break;
    }
    case Node_Types::FunctionDeclarationNode:
    {
      Function_Declaration_Node *node = std::get<Function_Declaration_Node *>(node->function_declaration_node);
      cout << "Function Declaration Node" << endl;
      cout << "PARAMS: ";
      for (int i = 0; i < node->parameters.size(); i++)
      {
        cout << i + 1 << endl;
      }
      break;
    }
    default:
      break;
    }
  }
}