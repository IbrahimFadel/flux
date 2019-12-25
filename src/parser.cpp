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
  print_node.parameter = tokens[i + 2];

  return print_node;
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