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

void create_while_node(vector<Token> tokens, int i)
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

  cout << condition.left.value << endl;
  cout << condition.right.value << endl;
}

void generate_ast(vector<Token> tokens)
{
  for (int i = 0; i < tokens.size(); i++)
  {
    Token token = tokens[i];
    cout << token.type << " => " << token.value << endl;
    if (token.type == Types::kw)
    {
      if (token.value == "while")
      {
        cout << "hi" << endl;
        create_while_node(tokens, i);
      }
    }
  }
}