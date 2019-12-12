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

// ! keep track of tokens parsed so that - print parsed inside the while loop isn't parsed again later
// ! if you're parsing a while loop, skip to the end of the  closing squiggly bracket after
vector<WhileNode> while_nodes;

PrintNode create_print_node(vector<Token> tokens, int i)
{
  PrintNode node;
  // Convert tokens[i].value to a string -- if it doesn't work throw error else, turn it into a token and return the node
  node.print_value = create_token(Types::lit, tokens[i].value);

  return node;
}

void check_token(vector<Token> tokens, int i)
{
  for (int j = 0; j < while_nodes.size(); j++)
  {
    // cout << while_nodes[j].then.end_position << endl;
    if (while_nodes[j].then.end_position == i)
    {
      return;
    }
  }
  // generic function that's run in generate_ast, and inside the inside of while loopes/if statements etc.
  Token token = tokens[i];
  // cout << token.value << endl;
  if (token.type == Types::kw)
  {
    if (token.value == "while")
    {
      WhileNode node = create_while_node(tokens, i);
      while_nodes.push_back(node);
    }
    else if (token.value == "print")
    {
      PrintNode node = create_print_node(tokens, i);
      cout << node.print_value.value << endl;
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
          cout << j << "hello" << endl;
          then.end_position = j;
          return then;
        }
      }
    }

    then.tokens.push_back(token);
  }

  if (open_curly_brackets == 0 && closed_curly_brackets == 1)
  {
    then.tokens.pop_back();

    // for (int j = i + 1; j < tokens.size(); j++)
    // {
    // if (tokens[j].value == "}" && tokens[j].type == Types::sep)
    // {
    // then.end_position = j;
    // }
    // }

    // cout << then.end_position << endl;

    for (int j = 0; j < then.tokens.size(); j++)
    {
      check_token(then.tokens, j);
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
    check_token(tokens, i);
  }
}