#include <iostream>
#include "lexer.h"
#include "parser.h"

using namespace Lexer;
using namespace Parser;

using std::cout;
using std::endl;
using std::string;
using std::vector;

Tree ast;

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
  int close_curly_brackets = 0;
  vector<Token> then_tokens;
  for (int j = i + 6; j < tokens.size(); j++)
  {
    if (tokens[j].value == "{" && tokens[j].type == Types::sep)
    {
      open_curly_brackets++;
    }
    else if (tokens[j].value == "}" && tokens[j].type == Types::sep)
    {
      close_curly_brackets++;
    }
    then_tokens.push_back(tokens[j]);

    if (open_curly_brackets == close_curly_brackets && open_curly_brackets != 0)
    {
      break;
    }
  }

  if (open_curly_brackets != close_curly_brackets)
  {
    std::cerr << "Are you missing a '}' ?" << endl;
    return while_node;
  }

  Position end_position;
  end_position.line_number = then_tokens[then_tokens.size() - 1].line_number;
  end_position.line_position = then_tokens[then_tokens.size() - 1].line_position;
  while_node.then.end = end_position;

  for (int j = 0; j < then_tokens.size(); j++)
  {
    // cout << then_tokens[j].value << endl;
    Node node = check_token(then_tokens, j);
    while_node.then.nodes.push_back(node);
    // cout << node.type << ' ' << then_tokens[j].value << endl;
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
  int close_curly_brackets = 0;
  vector<Token> then_tokens;
  for (int j = i + 6; j < tokens.size(); j++)
  {
    if (tokens[j].value == "{" && tokens[j].type == Types::sep)
    {
      open_curly_brackets++;
    }
    else if (tokens[j].value == "}" && tokens[j].type == Types::sep)
    {
      close_curly_brackets++;
    }

    then_tokens.push_back(tokens[j]);

    if (open_curly_brackets == close_curly_brackets && open_curly_brackets != 0)
    {
      break;
    }
  }

  if (open_curly_brackets != close_curly_brackets)
  {
    std::cerr << "You fucking donkey. You forgot a '}' ?" << endl;
    return if_node;
  }

  Position end_position;
  end_position.line_number = then_tokens[then_tokens.size() - 1].line_number;
  end_position.line_position = then_tokens[then_tokens.size() - 1].line_position;
  if_node.then.end = end_position;

  for (int j = 0; j < then_tokens.size(); j++)
  {
    // cout << then_tokens[j].value << endl;
    Node node = check_token(then_tokens, j);
    if_node.then.nodes.push_back(node);
    // cout << node.type << ' ' << then_tokens[j].value << endl;
  }

  // cout << "hello" << endl;
  // cout << open_curly_brackets << " " << close_curly_brackets << endl;
  return if_node;
}

Node check_token(vector<Token> tokens, int i)
{
  Node node;

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

  cout << tokens[i].value << ' ' << tokens[i].type << endl;
  if (tokens[i].type == Types::kw)
  {
    if (tokens[i].value == "while")
    {
      node = create_while_node(tokens, i);
    }
    else if (tokens[i].value == "if")
    {
      node = create_if_node(tokens, i);
      // cout << "hi there" << endl;
    }
    else
    {
      node.type = -1;
    }
  }
  else
  {
    node.type = -1;
  }

  return node;
}

void generate_ast(vector<Token> tokens)
{
  for (int i = 0; i < tokens.size(); i++)
  {
    Token token = tokens[i];

    Node node = check_token(tokens, i);
    ast.nodes.push_back(node);
  }

  // cout << ast.nodes[0].condition.left.value << endl;
  // cout << ast.nodes[0].condition.op.value << endl;
  // cout << ast.nodes[0].condition.right.value << endl;
  // cout << endl;
  // for (int i = 0; i < ast.nodes[0].then.nodes.size(); i++)
  // {
  //   Node node = ast.nodes[0].then.nodes[i];
  //   cout << ' ' << node.type << endl;
  // }

  // for (int i = 0; i < nodes.size(); i++)
  // {
  //   cout << nodes[i].type << endl;
  // }
}