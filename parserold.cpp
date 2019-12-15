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
vector<PrintNode> print_nodes;

PrintNode create_print_node(vector<Token> tokens, int i)
{
  PrintNode node;
  for (int j = i + 2; j < tokens.size() - i; j++)
  {
    if (tokens[j].type == Types::lit)
    {
      node.print_value = tokens[j].value;
    }
  }

  return node;
}

void check_token(vector<Token> tokens, int i)
{
  Token token = tokens[i];

  for (int j = 0; j < while_nodes.size(); j++)
  {
    if (token.line_number < while_nodes[j].start_position.line_number)
    {
      return;
    }
    else if (token.line_number <= while_nodes[j].start_position.line_number && token.line_position < while_nodes[j].start_position.line_position + 1)
    {
      return;
    }
    if (token.line_number < while_nodes[j].then.end_position.line_number)
    {
      return;
    }
    else if (token.line_number <= while_nodes[j].then.end_position.line_number && token.line_position < while_nodes[j].then.end_position.line_position)
    {
      return;
    }
  }

  cout << token.value << endl;

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
      print_nodes.push_back(node);
    }
  }
}

Then create_then_Node(vector<Token> tokens, int i)
{
  Then then;

  int open_curly_brackets = 1;
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
        cout << open_curly_brackets << ' ' << closed_curly_brackets << endl;
        if (closed_curly_brackets == open_curly_brackets)
        {
          Position pos;
          pos.line_number = token.line_number;
          pos.line_position = token.line_position;
          then.end_position = pos;
          cout << "end " << then.tokens.size() << endl;
          return then;
        }
      }
    }

    // cout << token.value << endl;
    then.tokens.push_back(token);
    cout << then.tokens.size() << endl;
  }

  if (open_curly_brackets == 0 && closed_curly_brackets == 1)
  {
    then.tokens.pop_back();

    Position pos;
    pos.line_number = then.tokens[then.tokens.size()].line_number;
    pos.line_position = then.tokens[then.tokens.size()].line_position;
    then.end_position = pos;

    for (int j = 0; j < then.tokens.size(); j++)
    {
      check_token(then.tokens, j);
    }

    cout << "end " << then.tokens.size() << endl;
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
      Position pos;
      pos.line_number = tokens[j].line_number;
      pos.line_position = tokens[j].line_position;
      while_node.start_position = pos;
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

  // for (int i = 0; i < while_nodes.size(); i++)
  // {
  //   cout << "--------------- WHILE NODE ---------------" << endl;
  //   cout << "---- CONDITION ----" << endl;
  //   cout << while_nodes[i].condition.left.value << ' ' << while_nodes[i].condition.operator_node.value << ' ' << while_nodes[i].condition.right.value << endl;
  //   cout << "---- THEN ----" << endl;
  //   cout << while_nodes[i].then.tokens.size() << endl;
  //   for (int j = 0; j < while_nodes[i].then.tokens.size(); j++)
  //   {
  //     cout << while_nodes[i].then.tokens[j].value << endl;
  //   }
  // }

  // for (int i = 0; i < print_nodes.size(); i++)
  // {
  //   cout << "--------------- PRINT NODE ---------------" << endl;
  //   cout << print_nodes[i].print_value << endl;
  // }
}