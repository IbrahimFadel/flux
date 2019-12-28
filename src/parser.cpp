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

Node create_while_node(vector<Token> tokens, int i)
{
  Node while_node;
  while_node.type = Node_Types::_while;
  while_node.condition.left = tokens[i + 2];
  while_node.condition.op = tokens[i + 3];
  while_node.condition.right = tokens[i + 4];

  Position start_position;
  start_position.line_number = tokens[i + 6].line_number;
  start_position.line_position = tokens[i + 6].line_position;
  while_node.then.start = start_position;

  int open_curly_brackets = 0;
  int closed_curly_brackets = 0;
  vector<Token> then_tokens;
  for (int j = i + 1; j < tokens.size(); j++)
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

  if (open_curly_brackets != closed_curly_brackets)
  {
    std::cerr << "Are you missing a '}' ?" << endl;
    return while_node;
  }

  while_node.then.tokens = then_tokens;

  Position end_position;
  end_position.line_number = then_tokens[then_tokens.size() - 1].line_number;
  end_position.line_position = then_tokens[then_tokens.size() - 1].line_position;
  while_node.then.end = end_position;

  Node node;
  int closed_curly_brackets_found = 0;
  int skip = 0;
  int skipped = 0;

  /**
   *  Make sure we skip over the tokens inside the 'Then' of any nodes it may find
   *  Those tokens should be children of the node itself
  */
  for (int j = 0; j < then_tokens.size(); j++)
  {
    for (int x = 0; x < skip; x++)
    {
      if (skipped + 1 == skip)
      {
        skipped = 0;
        skip = 0;
        goto end;
      }
      goto end;
      skipped++;
    }
    node = check_token(then_tokens, j, &while_node);
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
  if_node.condition.left = tokens[i + 2];
  if_node.condition.op = tokens[i + 3];
  if_node.condition.right = tokens[i + 4];

  int open_curly_brackets = 0;
  int closed_curly_brackets = 0;
  vector<Token> then_tokens;
  for (int j = i + 6; j < tokens.size(); j++)
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

  if (open_curly_brackets != closed_curly_brackets)
  {
    std::cerr << "You fucking donkey. You forgot a '}' ?" << endl;
    return if_node;
  }

  if_node.then.tokens = then_tokens;

  Position end_position;
  end_position.line_number = then_tokens[then_tokens.size() - 1].line_number;
  end_position.line_position = then_tokens[then_tokens.size() - 1].line_position;
  if_node.then.end = end_position;

  int closed_curly_brackets_found = 0;
  for (int j = 0; j < then_tokens.size(); j++)
  {
    Node node = check_token(then_tokens, j, &if_node);
    if_node.then.nodes.push_back(node);
  }
  return if_node;
}

Node create_print_node(vector<Token> tokens, int i)
{
  Node print_node;

  print_node.type = Node_Types::print;
  print_node.print_value = tokens[i + 2].value;

  return print_node;
}

Node create_number_node(vector<Token> tokens, int i)
{
  Node number_node;

  number_node.type = Node_Types::number;
  number_node.number_value = std::stoi(tokens[i].value);

  return number_node;
}

Node create_string_node(vector<Token> tokens, int i)
{
  Node string_node;

  string_node.type = Node_Types::_string;
  string string_value = tokens[i].value;
  string_node.string_value = string_value;

  return string_node;
}

Node create_less_than_node(vector<Token> tokens, int i)
{
  Node less_than_node;

  less_than_node.type = Node_Types::op;
  less_than_node.op = "<";

  return less_than_node;
}

Node create_greater_than_node(vector<Token> tokens, int i)
{
  Node greater_than_node;

  greater_than_node.type = Node_Types::op;
  greater_than_node.op = ">";

  return greater_than_node;
}

Node create_equals_node(vector<Token> tokens, int i)
{
  Node equals_node;

  equals_node.type = Node_Types::op;
  equals_node.op = "=";

  return equals_node;
}

Node create_open_parentheses_node(vector<Token> tokens, int i)
{
  Node open_parentheses_node;

  open_parentheses_node.type = Node_Types::sep;
  open_parentheses_node.sep = "(";

  return open_parentheses_node;
}

Node create_closed_parentheses_node(vector<Token> tokens, int i)
{
  Node closed_parentheses_node;

  closed_parentheses_node.type = Node_Types::sep;
  closed_parentheses_node.sep = ")";

  return closed_parentheses_node;
}

Node create_open_curly_bracket_node(vector<Token> tokens, int i)
{
  Node open_curly_bracket_node;

  open_curly_bracket_node.type = Node_Types::sep;
  open_curly_bracket_node.sep = "{";

  return open_curly_bracket_node;
}

Node create_closed_curly_bracket_node(vector<Token> tokens, int i)
{
  Node closed_curly_bracket_node;

  closed_curly_bracket_node.type = Node_Types::sep;
  closed_curly_bracket_node.sep = "}";

  return closed_curly_bracket_node;
}

Node create_eol_node(vector<Token> tokens, int i)
{
  Node eol_node;

  eol_node.type = Node_Types::eol;

  return eol_node;
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
      node.skip = node.then.tokens.size();
    }
    else if (tokens[i].value == "if")
    {
      node = create_if_node(tokens, i);
      node.skip = node.then.tokens.size();
    }
    else if (tokens[i].value == "print")
    {
      node = create_print_node(tokens, i);
    }
    else
    {
      node.type = -1;
    }
  }
  else if (tokens[i].type == Types::lit)
  {
    if (is_number(tokens[i].value))
    {
      node = create_number_node(tokens, i);
    }
    else if (tokens[i].value.substr(0, 1) == "\"")
    {
      node = create_string_node(tokens, i);
    }
  }
  else if (tokens[i].type == Types::op)
  {
    if (tokens[i].value == "<")
    {
      node = create_less_than_node(tokens, i);
    }
    else if (tokens[i].value == ">")
    {
      node = create_greater_than_node(tokens, i);
    }
    else if (tokens[i].value == "=")
    {
      node = create_equals_node(tokens, i);
    }
  }
  else if (tokens[i].type == Types::sep)
  {
    if (tokens[i].value == "(")
    {
      node = create_open_parentheses_node(tokens, i);
    }
    else if (tokens[i].value == ")")
    {
      node = create_closed_parentheses_node(tokens, i);
    }
    else if (tokens[i].value == "{")
    {
      node = create_open_curly_bracket_node(tokens, i);
    }
    else if (tokens[i].value == "}")
    {
      node = create_closed_curly_bracket_node(tokens, i);
    }
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