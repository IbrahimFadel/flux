#include <iostream>
#include <ctype.h>
#include "lexer.h"
#include "parser.h"

using namespace Lexer;
using namespace Parser;

using std::cout;
using std::endl;
using std::string;
using std::vector;

Tree ast;

bool is_number(const std::string &s)
{
  std::string::const_iterator it = s.begin();
  while (it != s.end() && std::isdigit(*it))
    ++it;
  return !s.empty() && it == s.end();
}

Condition get_condition(vector<Token> tokens, int i)
{
  Condition condition;

  vector<Token> lefts;
  vector<Token> ops;
  vector<Token> rights;
  vector<Token> condition_seperators;
  vector<Token> results_operators;
  vector<Token> results;
  int condition_counter = 0;

  int skip_conditions = 0;
  int skipped_conditions = 0;

  for (int j = i + 2; j < tokens.size(); j++)
  {
    for (int x = 0; x < skip_conditions; x++)
    {
      if (skipped_conditions + 1 == skip_conditions)
      {
        skipped_conditions = 0;
        skip_conditions = 0;
        goto end_conditions;
      }
      skipped_conditions++;
      goto end_conditions;
    }

    if (tokens[j].value == "&&" || tokens[j].value == "||")
    {
      condition_counter = 0;
      condition_seperators.push_back(tokens[j]);
      continue;
    }
    if (tokens[j].value == ")" && tokens[j].type == Types::sep)
    {
      break;
    }

    if (condition_counter % 3 == 0)
    {
      lefts.push_back(tokens[j]);
    }
    else if (condition_counter % 2 == 0)
    {
      rights.push_back(tokens[j]);
      if (tokens[j + 1].value == "==")
      {
        results_operators.push_back(tokens[j + 1]);
        results.push_back(tokens[j + 2]);
        skip_conditions = 2;
      }
    }
    else if (condition_counter % 1 == 0)
    {
      ops.push_back(tokens[j]);
    }

    condition_counter++;

  end_conditions:;
  }

  condition.lefts = lefts;
  condition.ops = ops;
  condition.rights = rights;
  condition.condition_seperators = condition_seperators;
  condition.results = results;
  condition.results_operators = results_operators;

  return condition;
}

Then get_then(vector<Token> tokens, int i, Condition condition)
{
  Then then;

  int open_curly_brackets = 0;
  int closed_curly_brackets = 0;
  vector<Token> then_tokens;
  int start_index = i + 3 + condition.lefts.size() + condition.rights.size() + condition.ops.size() + condition.condition_seperators.size() + condition.results_operators.size() + condition.results.size();
  for (int j = start_index; j < tokens.size(); j++)
  {
    if (tokens[j].value == "{" && tokens[j].type == Types::sep)
    {
      open_curly_brackets++;
    }
    else if (tokens[j].value == "}" && tokens[j].type == Types::sep)
    {
      closed_curly_brackets++;
    }

    then_tokens.push_back(tokens[j]);

    if (open_curly_brackets == closed_curly_brackets && open_curly_brackets != 0)
    {
      break;
    }
  }

  then.tokens = then_tokens;

  Position end_position;
  end_position.line_number = then_tokens[then_tokens.size() - 1].line_number;
  end_position.line_position = then_tokens[then_tokens.size() - 1].line_position;
  then.end = end_position;

  if (open_curly_brackets != closed_curly_brackets)
  {
    std::cerr << "Are you missing a '}' ?" << endl;
    return then;
  }

  return then;
}

Node create_while_node(vector<Token> tokens, int i)
{
  Node while_node;
  while_node.type = Node_Types::_while;

  Condition condition = get_condition(tokens, i);
  while_node.condition = condition;

  Then then = get_then(tokens, i, condition);
  while_node.then = then;

  Node node;
  int closed_curly_brackets_found = 0;
  int skip = 0;
  int skipped = 0;
  for (int j = 0; j < then.tokens.size(); j++)
  {
    for (int x = 0; x < skip; x++)
    {
      if (skipped + 1 == skip)
      {
        skipped = 0;
        skip = 0;
        goto end;
      }
      skipped++;
      goto end;
    }
    node = check_token(then.tokens, j, &while_node);
    while_node.then.nodes.push_back(node);
    skip = node.skip;

  end:;
  }

  return while_node;
}

Node create_if_node(vector<Token> tokens, int i)
{
  Node if_node;
  if_node.type = Node_Types::_if;

  Condition condition = get_condition(tokens, i);
  if_node.condition = condition;

  Then then = get_then(tokens, i, condition);
  if_node.then = then;

  Node node;
  int closed_curly_brackets_found = 0;
  int skip = 0;
  int skipped = 0;
  for (int j = 0; j < then.tokens.size(); j++)
  {
    for (int x = 0; x < skip; x++)
    {
      if (skipped + 1 == skip)
      {
        skipped = 0;
        skip = 0;
        goto end;
      }
      skipped++;
      goto end;
    }
    node = check_token(then.tokens, j, &if_node);
    if_node.then.nodes.push_back(node);
    skip = node.skip;

  end:;
  }
  return if_node;
}

Node create_number_node(vector<Token> tokens, int i)
{
  Node number_node;

  number_node.type = Node_Types::number;
  number_node.number_value = std::stoi(tokens[i].value);

  return number_node;
}

Node create_operator_node(vector<Token> tokens, int i)
{
  Node operator_node;

  operator_node.type = Node_Types::op;
  operator_node.op = tokens[i].value;

  return operator_node;
}

Node create_seperator_node(vector<Token> tokens, int i)
{
  Node seperator_node;

  seperator_node.type = Node_Types::sep;
  seperator_node.sep = tokens[i].value;

  return seperator_node;
}

Node create_eol_node(vector<Token> tokens, int i)
{
  Node eol_node;

  eol_node.type = Node_Types::eol;

  return eol_node;
}

Node create_literal_node(Token token)
{
  Node literal_node;

  literal_node.type = Node_Types::lit;

  if (token.value.substr(0, 1) == "\"" && token.type == Types::lit)
  {
    literal_node.string_value = token.value;
  }
  else if (is_number(token.value))
  {
    literal_node.number_value = std::stoi(token.value);
  }

  return literal_node;
}

Node create_identifier_node(Token token)
{
  Node identifier_node;

  identifier_node.type = Node_Types::id;
  identifier_node.id_name = token.value;

  return identifier_node;
}

Node create_function_call_node(vector<Token> tokens, int i)
{
  Node function_call_node;

  function_call_node.type = Node_Types::function_call;
  function_call_node.function_call_name = tokens[i].value;

  int comma = 0;
  Node param;
  vector<Node> parameters;
  for (int j = i + 2; j < tokens.size(); j++)
  {
    if (tokens[j].type == Types::sep && tokens[j].value == ")")
    {
      break;
    }
    if (comma % 2 == 0)
    {
      if (tokens[j].value.substr(0, 1) == "\"" || is_number(tokens[j].value))
      {
        param = create_literal_node(tokens[j]);
      }
      else
      {
        param = create_identifier_node(tokens[j]);
      }

      parameters.push_back(param);
    }
    comma++;
  }

  function_call_node.parameters = parameters;

  return function_call_node;
}

Node create_let_node(vector<Token> tokens, int i)
{
  Node let_node;

  let_node.type = Node_Types::let;
  let_node.variable_name = tokens[i + 1].value;

  Node parent;
  Node value = check_token(tokens, i + 3, &parent);

  if (value.type == Node_Types::lit)
  {
    if (value.string_value.substr(0, 1) == "\"")
    {
      String string_value;
      string_value.value = value.string_value;
      let_node.variable_value_string = string_value;
    }
    else
    {
      Number number_value;
      number_value.value = value.number_value;
      let_node.variable_value_number = number_value;
    }
  }

  return let_node;
}

Node create_assignment_node(vector<Token> tokens, int i)
{
  Node assignment_node;
  Node parent;

  assignment_node.type = Node_Types::assign;
  assignment_node.id_name = tokens[i].value;

  vector<Node> assignment_values;
  for (int j = i + 2; j < tokens.size(); j++)
  {
    if (tokens[j].type == Types::eol)
    {
      break;
    }
    Node val = check_token(tokens, j, &parent);
    assignment_values.push_back(val);
  }

  assignment_node.assignment_values = assignment_values;

  // assignment_node.skip = 2 + assignment_values.size() - 1;

  return assignment_node;
}

Node check_token(vector<Token> tokens, int i, Node *parent)
{
  Node node;
  node.parent = parent;

  /**
   * Make sure we don't look at tokens already checked(eg. inside a while loop or an if statement)
   */
  for (int j = 0; j < ast.nodes.size(); j++)
  {
    Node node = ast.nodes[j];
    if (node.then.end.line_number > tokens[i].line_number)
    {
      node.type = -1;
      return node;
    }
    else if (node.then.end.line_number == tokens[i].line_number && node.then.end.line_position >= tokens[i].line_position)
    {
      node.type = -1;
      return node;
    }
  }

  if (tokens[i].type == Types::kw)
  {
    if (tokens[i].value == "while")
    {
      node = create_while_node(tokens, i);
      node.skip = node.then.tokens.size() + 2 + node.condition.lefts.size() + node.condition.ops.size() + node.condition.rights.size() + node.condition.condition_seperators.size();
    }
    else if (tokens[i].value == "if")
    {
      node = create_if_node(tokens, i);
      node.skip = node.then.tokens.size() + 2 + node.condition.lefts.size() + node.condition.ops.size() + node.condition.rights.size() + node.condition.condition_seperators.size() + node.condition.results_operators.size() + node.condition.results.size();
    }
    else if (tokens[i].value == "print")
    {
      node = create_function_call_node(tokens, i);
      node.skip = node.parameters.size() + 3;
    }
    else if (tokens[i].value == "let")
    {
      node = create_let_node(tokens, i);
      node.skip = 4;
    }
  }
  else if (tokens[i].type == Types::lit)
  {
    node = create_literal_node(tokens[i]);
  }
  else if (tokens[i].type == Types::op)
  {
    node = create_operator_node(tokens, i);
  }
  else if (tokens[i].type == Types::id)
  {
    if (tokens[i + 1].value == "=")
    {
      node = create_assignment_node(tokens, i);
      node.skip = 2 + node.assignment_values.size() - 1;
    }
    else
    {
      node = create_identifier_node(tokens[i]);
    }
  }
  else if (tokens[i].type == Types::sep)
  {
    node = create_seperator_node(tokens, i);
  }
  else if (tokens[i].type == Types::eol)
  {
    node = create_eol_node(tokens, i);
  }
  else
  {
    node.type = -1;
  }

  return node;
}

Tree generate_ast(vector<Token> tokens)
{
  Token token;
  Node parent;
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

    node = check_token(tokens, i, &parent);
    ast.nodes.push_back(node);
    skip = node.skip;

  end:;
  }

  return ast;
}