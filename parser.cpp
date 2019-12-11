#include <iostream>
#include <vector>
#include <stdexcept>
#include <sstream>

#include "lexer.h"
#include "parser.h"

using namespace Lexer;
using namespace Parser;

using std::cout;
using std::endl;
using std::string;
using std::vector;

void check_tokens(vector<Token> tokens, int i)
{
  // generic function that's run in generate_ast, and inside the inside of while loopes/if statements etc.
  Token token = tokens[i];
  if (token.type == Types::kw)
  {
    if (token.value == "while")
    {
      WhileNode node = create_while_node(tokens, i);
      for (int j = 0; j < node.then.tokens.size(); j++)
      {
        cout << node.then.tokens[j].value << endl;
      }
    }
  }
}

Then create_then_Node(vector<Token> tokens, int i)
{
  Then then;

  int open_curly_brackets = 0;
  int closed_curly_brackets = 0;
  for (int j = i + 1; j < tokens.size(); j++)
  {
    Token token = tokens[j];
    if (token.type == Types::sep)
    {
      if (token.value == "{")
      {
        open_curly_brackets++;
      }
      else if (token.value == "}")
      {
        closed_curly_brackets++;

        if (closed_curly_brackets == open_curly_brackets)
        {
          return then;
        }
      }
    }

    then.tokens.push_back(token);
  }

  if (open_curly_brackets == 0 && closed_curly_brackets == 1)
  {
    then.tokens.pop_back();

    for (int j = 0; j < then.tokens.size(); j++)
    {
      check_tokens(then.tokens, j);
    }

    return then;
  }

  string err_msg;
  err_msg = "Did you forget a } in your while loop?";
  throw std::invalid_argument(err_msg);
}

Condition create_condition(vector<Token> tokens, int i)
{
  NumberNode left;
  left.value = std::stoi(tokens[i].value);
  NumberNode right;
  right.value = std::stoi(tokens[i + 2].value);
  OperatorNode op;
  op.value = tokens[i + 1].value;

  Condition condition;
  condition.left = left;
  condition.right = right;
  condition.operator_node = op;

  return condition;
}

WhileNode create_while_node(vector<Token> tokens, int i)
{
  std::ostringstream oss;
  string err_msg;
  if (tokens[i + 1].type != Types::sep && tokens[i + 1].value != "(")
  {

    oss << "Invalid token " << tokens[i + 1].value << " Expected '(' ";
    err_msg = oss.str();
    throw std::invalid_argument(err_msg);
  }

  WhileNode while_node;
  Condition condition = create_condition(tokens, i + 2);
  Then then;
  for (int j = i; j < tokens.size() - i; j++)
  {
    if (tokens[j].value == "{")
    {
      then = create_then_Node(tokens, j);
    }
  }

  while_node.condition = condition;
  while_node.then = then;
  return while_node;
}

void generate_ast(vector<Token> tokens)
{
  for (int i = 0; i < tokens.size(); i++)
  {
    check_tokens(tokens, i);
  }
}