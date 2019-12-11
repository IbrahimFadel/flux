#include <iostream>
#include "lexer.h"
#include "parser.h"

using namespace Parser;
using namespace Lexer;
using std::cout;
using std::endl;
using std::vector;

void print_expression(Expression expr)
{
  // cout << '(' << expr.left.factors[0].value << ' ' << expr.left.operator_token.value << ' ' << expr.left.factors[1].value << ')' << ' ' << expr.operator_token.value << ' ' << '(' << expr.right.factors[0].value << ' ' << expr.right.operator_token.value << ' ' << expr.right.factors[1].value << ')' << endl;
}

int Term::evaluate()
{
  int value = 0;
  for(int i = 0; i < factors.size(); i++)
  {
    // cout << factors[i].value << ' ';
    if(i < factors.size() - 1)
    {
      if(operator_tokens[i].value == "+")
      {
        value += factors[i].value;
      }
      else
      {
        value -= factors[i].value;
      }
    }
    else
    {
      if(operator_tokens[i - 1].value == "+")
      {
        value += factors[i].value;
      }
      else
      {
        value -= factors[i].value;
      }
    }
  }
  // cout << endl;

  for(int i = 0; i < operator_tokens.size(); i++)
  {
    // cout << operator_tokens[i].value << ' ';
  }
  // cout << endl;

  // cout << value << endl;
  return value;
}

Term term(vector<Token> tokens, int i, bool left)
{
  Term new_term;
  if(!left)
  {
    for(int j = i; j < tokens.size(); j++)
    {
      if(tokens[j].type == Types::op && (tokens[j].value == "*" || tokens[j].value == "/"))
      {
        break;
      }
      if(tokens[j].type == Types::num)
      {
        Factor new_factor;
        new_factor.value = std::stoi(tokens[j].value);
        new_term.factors.push_back(new_factor);

        if(tokens[j + 1].type == Types::op && (tokens[j + 1].value == "+" || tokens[j + 1].value == "-") && j + 1 < tokens.size())
        {
          cout << tokens[j+1].value << endl;
          Token new_operator_token;
          new_operator_token.type = Types::op;
          new_operator_token.value = tokens[j + 1].value;
          new_term.operator_tokens.push_back(new_operator_token);
        }
        else if(new_term.factors.size() < 2 && j + 2 >= tokens.size())
        {
          cout << "hi" << endl;
          // cout << tokens[j-1].value << endl;
          Token new_operator_token;
          new_operator_token.type = Types::op;
          new_operator_token.value = tokens[j - 1].value;
          new_term.operator_tokens.push_back(new_operator_token);
        }
      }
    }
  } else
  {
    for(int j = i; j >= 0; j--)
    {
      if(tokens[j].type == Types::op && (tokens[j].value == "*" || tokens[j].value == "/"))
      {
        break;
      }
      if(tokens[j].type == Types::num)
      {
        Factor new_factor;
        new_factor.value = std::stoi(tokens[j].value);
        new_term.factors.push_back(new_factor);

        if(tokens[j - 1].type == Types::op && (tokens[j - 1].value == "+" || tokens[j - 1].value == "-") && j - 1 > 0)
        {
          Token new_operator_token;
          new_operator_token.type = Types::op;
          new_operator_token.value = tokens[j - 1].value;
          new_term.operator_tokens.push_back(new_operator_token);
        }
      }
    }
  }

  // for(int j = 0; j < new_term.factors.size(); j++)
  // {
  //   cout << new_term.factors[j].value << ' ';
  // }
  // for(int j = 0; j < new_term.operator_tokens.size(); j++)
  // {
  //   cout << new_term.operator_tokens[j].value << ' ';
  // }
  // cout << endl;

  return new_term;
}

Expression expression(vector<Token> tokens, int i)
{
  if (tokens[i].value == "*" || tokens[i].value == "/")
  {
    Term left = term(tokens, i - 1, true);
    Term right = term(tokens, i + 1, false);
    Token operator_token;
    operator_token.value = tokens[i].value;
    operator_token.type = Types::op;
    Expression expr;
    expr.left = left;
    expr.right = right;
    expr.operator_token = operator_token;
    return expr;
  }
}

Expression another_expression(vector<Token> tokens, int i, Expression other_expr)
{
  if (tokens[i].value == "*" || tokens[i].value == "/")
  {
    Term left;
    // for(int j = 0; i < other_expr.left.factors.size(); j++)
    // {
      // left.factors.push_back(other_expr.left.factors[j]);
    // }
    Term right = term(tokens, i + 1, false);
    Token operator_token;
    operator_token.value = tokens[i].value;
    operator_token.type = Types::op;
    Expression expr;
    expr.left = left;
    expr.right = right;
    expr.operator_token = operator_token;
    return expr;
  }
}

/*

1 + 3 * 2 + 1 + 6

Expr:   Term (MUL|DIV) Term
Term:   Factor (ADD|SUB) Factor
Factor: (INT|FLT)

*/

void generate_ast(vector<Token> tokens)
{

  vector<Expression> expressions;
  int last_time = 0;
  for (int i = 0; i < tokens.size(); i++)
  {
    Token tok = tokens[i];
    if (tok.type == Types::op && (tok.value == "*" || tok.value == "/"))
    {
      if(last_time > 0)
      {
        Expression expr = another_expression(tokens, i, expressions[i - 1]);
        expressions.push_back(expr);
      } else
      {
        // cout << "h" << endl;
        Expression expr = expression(tokens, i);
        expressions.push_back(expr);
      }
      last_time++;
    }
  }

  expressions[0].left.evaluate();
  expressions[0].right.evaluate();
  // cout << expressions[0].left.evaluate() << endl;
  // cout << expressions[0].right.evaluate() << endl;
}
