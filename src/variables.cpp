#include "variables.h"

int Variables::get_precedence(char op){
    if(op == '+'||op == '-')
    return 1;
    if(op == '*'||op == '/')
    return 2;
    return 0;
}

int Variables::apply_operation(int a, int b, char op)
{
  switch(op){
        case '+': return a + b;
        case '-': return a - b;
        case '*': return a * b;
        case '/': return a / b;
    }
}

/**
 * <3 <3 <3 <3
 * https://www.geeksforgeeks.org/expression-evaluation/
 * <3 <3 <3 <3
 * I tried this so many times on my own but it kept getting so messy and horrible
 * This was a great solution
*/
Variables::Expression Variables::evaluate_expression(std::vector<Lexer::Token> tokens, int i)
{
  Variables::Expression expression;
  std::stack<int> values;
  std::stack<char> ops;

  for(int x = i; x < tokens.size(); x++)
  {
    if(tokens[x].type == Lexer::Token_Types::eol)
    {
      while(!ops.empty())
      {
        int val2 = values.top();
        values.pop();
        int val1 = values.top();
        values.pop();
        char op = ops.top();
        ops.pop();
        values.push(Variables::apply_operation(val1, val2, op));
      }
      expression.int_value = values.top();
      return expression;
    }
    else if(tokens[x].type == Lexer::Token_Types::sep)
    {
      if(tokens[x].value == "(")
      {
        ops.push(tokens[x].value.c_str()[0]);
      }
      else if(tokens[x].value == ")")
      {
        while(!ops.empty() && ops.top() != '(')
        {
          int val2 = values.top();
          values.pop();

          int val1 = values.top();
          values.pop();

          char op = ops.top();
          ops.pop();

          values.push(Variables::apply_operation(val1, val2, op));
        }

        if(!ops.empty())
          ops.pop();
      }
    }
    else if(tokens[x].type == Lexer::Token_Types::op)
    {
      while(!ops.empty() && Variables::get_precedence(ops.top()) >= Variables::get_precedence(tokens[x].value.c_str()[0]))
      {
        int val2 = values.top();
        values.pop();

        int val1 = values.top();
        values.pop();

        char op = ops.top();
        ops.pop();

        values.push(Variables::apply_operation(val1, val2, op));
      }

      ops.push(tokens[x].value.c_str()[0]);
    }
    else
    {
      values.push(std::stoi(tokens[x].value));
    }

    expression.skip++;
  }

  while(!ops.empty()){
    int val2 = values.top();
    values.pop();
    int val1 = values.top();
    values.pop();
    char op = ops.top();
    ops.pop();
    values.push(Variables::apply_operation(val1, val2, op));
  }

  expression.int_value = values.top();
  return expression;
}
