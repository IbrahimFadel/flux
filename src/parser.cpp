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
      node->skip = constant_declaration_node->skip;
      constants[constant_declaration_node->name] = constant_declaration_node;
    }
    else if (tokens[i]->value == "let")
    {
      Variable_Declaration_Node *variable_declaration_node = create_variable_declaration_node(tokens, i);
      node->type = Node_Types::VariableDeclarationNode;
      node->variable_declaration_node = variable_declaration_node;
      node->skip = variable_declaration_node->skip;
      variables[variable_declaration_node->name] = variable_declaration_node;
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

  node->skip = node->then.tokens.size() + then_start;

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

  int skip = 5;
  Number_Expression_Node num_node = std::get<Number_Expression_Node>(node->expression->number_expression);
  for (auto &term : num_node.terms)
  {
    for (auto &num : term.numbers)
    {
      skip += 1;
    }
    for (auto &op : term.ops)
    {
      skip += 1;
    }
  }
  for (auto &op : num_node.ops)
  {
    skip += 1;
  }

  node->skip = skip;

  return node;
}

std::unique_ptr<Expression_Node> create_expression_node(std::vector<std::shared_ptr<Token>> tokens, int i)
{
  std::unique_ptr<Expression_Node> expr_node = std::make_unique<Expression_Node>();
  expr_node->type = Expression_Types::NumberExpression;
  int x = i;
  Term term;
  Number_Expression_Node number_expression;
  while (tokens[x]->value != ";")
  {
    std::shared_ptr<Token> tok = tokens[x];
    if (tok->type == Token_Types::op)
    {
      if (tok->value == "+" || tok->value == "-")
      {
        number_expression.ops.push_back(tok->value);
        number_expression.terms.push_back(term);
        term.numbers.clear();
        term.ops.clear();
      }
      else
      {
        term.ops.push_back(tok->value);
      }
    }
    else if (tok->type == Token_Types::lit) //! When implementing variables, || Token_Types::id
    {
      term.numbers.push_back(tok->value);
      if (tokens[x + 1]->value == ";" || tokens[x + 1]->value == ")")
      {
        number_expression.terms.push_back(term);
      }
    }
    x++;
  }

  expr_node->number_expression = number_expression;

  return expr_node;
};

Variable_Declaration_Node *create_variable_declaration_node(std::vector<std::shared_ptr<Token>> tokens, int i)
{
  Variable_Declaration_Node *node = new Variable_Declaration_Node();

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

  int skip = 5;
  Number_Expression_Node num_node = std::get<Number_Expression_Node>(node->expression->number_expression);
  for (auto &term : num_node.terms)
  {
    for (auto &num : term.numbers)
    {
      skip += 1;
    }
    for (auto &op : term.ops)
    {
      skip += 1;
    }
  }
  for (auto &op : num_node.ops)
  {
    skip += 1;
  }

  node->skip = skip;

  return node;
}

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
      Function_Declaration_Node *function_declaration_node = std::get<Function_Declaration_Node *>(node->function_declaration_node);
      cout << "Function Declaration: " << function_declaration_node->name << endl;
      cout << "PARAMS: " << endl;

      std::map<std::string, Variable_Types>::iterator it;
      for (it = function_declaration_node->parameters.begin(); it != function_declaration_node->parameters.end(); it++)
      {
        cout << it->first << ": " << it->second << endl;
      }

      cout << "Return: " << function_declaration_node->return_type << endl;

      break;
    }
    case Node_Types::VariableDeclarationNode:
    {
      Variable_Declaration_Node *variable_declaration_node = std::get<Variable_Declaration_Node *>(node->variable_declaration_node);
      cout << "Variable Declaration Node: " << variable_declaration_node->name << endl;
      break;
    }
    default:
      break;
    }
  }

  cout << "-------------" << endl;
}