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
  }

  if (open_curly_brackets != close_curly_brackets)
  {
    std::cerr << "Are you missing a '}' ?" << endl;
  }
  else
  {
    Position end_position;
    end_position.line_number = then_tokens[then_tokens.size() - 1].line_number;
    end_position.line_position = then_tokens[then_tokens.size() - 1].line_position;
    while_node.then.end = end_position;

    while_node.then.tokens = then_tokens;
  }

  return while_node;
}

void check_token(vector<Token> tokens, int i)
{
  if (tokens[i].type == Types::kw)
  {
    if (tokens[i].value == "while")
    {
      Node while_node = create_while_node(tokens, i);
      cout << "------- CONDITION -------" << endl;
      cout << while_node.condition.left.value << " " << while_node.condition.op.value << " " << while_node.condition.right.value << endl;
      cout << "------- END CONDITION ------" << endl << endl;
      cout << "------- THEN -------" << endl << endl;
      for (int j = 0; j < while_node.then.tokens.size(); j++)
      {
        cout << while_node.then.tokens[j].value << endl;
      }
      cout << endl;
      cout << "START POSITION: " << "LN:" << while_node.then.start.line_number << " POS:" << while_node.then.start.line_position << endl;
      cout << "END POSITION: " << "LN:" << while_node.then.end.line_number << " POS:" << while_node.then.end.line_position << endl;
      cout << endl;
      cout << "------- END THEN -------" << endl;
    }
  }
}

void generate_ast(vector<Token> tokens)
{
  for (int i = 0; i < tokens.size(); i++)
  {
    Token token = tokens[i];

    check_token(tokens, i);
  }
}