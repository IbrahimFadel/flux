#include <iostream>
#include <map>
#include "interpreter.h"

using std::cout;
using std::endl;
using std::string;
using std::vector;

using Parser::Node_Types;

std::map<std::string, Variables::Variable> Variables::variables;
std::map<std::string, Variables::Variable>::iterator Variables::variables_it;

std::map<std::string, Variables::Function> Variables::functions;
std::map<std::string, Variables::Function>::iterator Variables::functions_it;
std::map<std::string, Variables::Variable>::iterator Variables::function_variables_it;

std::vector<Variables::If> Variables::ifs;
std::map<std::string, Variables::Variable>::iterator Variables::ifs_variables_it;

string evaluate_string_expression(Node node)
{
  string val = "";
  for (int i = 0; i < node.assignment_values.size(); i++)
  {
    if (node.assignment_values[i].string_value.length() > 0)
    {
      node.assignment_values[i].string_value = node.assignment_values[i].string_value.substr(1, node.assignment_values[i].string_value.length() - 2);
    }
  }
  for (int i = 0; i < node.assignment_values.size(); i++)
  {
    if (node.assignment_values.size() == 1)
    {
      return node.assignment_values[0].string_value;
    }
    if (node.assignment_values[i + 1].op == "+" || node.assignment_values[i + 1].op == "-")
    {
      if (val.length() == 0)
      {
        val = node.assignment_values[i].string_value + node.assignment_values[i + 2].string_value;
      }
      else
      {
        val = val + node.assignment_values[i + 2].string_value;
      }
    }
  }

  return val;
}

int evaluate_expression(Node node)
{
  vector<int> vals;
  for (int i = 0; i < node.assignment_values.size(); i++)
  {
    Node value = node.assignment_values[i];
    if (value.op == "+" || value.op == "-")
    {
      if (node.assignment_values[i + 2].op == "*" || node.assignment_values[i + 2].op == "/")
      {
        int a, b, c;
        if (node.assignment_values[i - 1].id_name.length() > 0)
        {
          Variables::variables_it = Variables::variables.find(node.assignment_values[i - 1].id_name);
          a = Variables::variables_it->second.number_value;
        }
        else
        {
          a = node.assignment_values[i - 1].number_value;
        }
        if (node.assignment_values[i + 1].id_name.length() > 0)
        {
          Variables::variables_it = Variables::variables.find(node.assignment_values[i + 1].id_name);
          b = Variables::variables_it->second.number_value;
        }
        else
        {
          b = node.assignment_values[i + 1].number_value;
        }
        if (node.assignment_values[i + 3].id_name.length() > 0)
        {
          Variables::variables_it = Variables::variables.find(node.assignment_values[i + 3].id_name);
          c = Variables::variables_it->second.number_value;
        }
        else
        {
          c = node.assignment_values[i + 3].number_value;
        }

        if (vals.size() > 0)
        {
          if (node.assignment_values[i + 2].op == "*")
          {
            if (value.op == "+")
            {
              vals[vals.size() - 1] = vals[vals.size() - 1] + b * c;
            }
            else
            {
              vals[vals.size() - 1] = vals[vals.size() - 1] - b * c;
            }
          }
          else
          {
            if (value.op == "+")
            {
              vals[vals.size() - 1] = vals[vals.size() - 1] + b / c;
            }
            else
            {
              vals[vals.size() - 1] = vals[vals.size() - 1] - b / c;
            }
          }

          continue;
        }
        int val;
        if (node.assignment_values[i + 2].op == "*")
        {
          if (value.op == "+")
          {
            val = a + b * c;
          }
          else
          {
            val = a - b * c;
          }
        }
        else
        {
          if (value.op == "+")
          {
            val = a + b / c;
          }
          else
          {
            val = a - b / c;
          }
        }

        vals.push_back(val);
      }
      else
      {
        int a, b;
        if (node.assignment_values[i - 1].id_name.length() > 0)
        {
          Variables::variables_it = Variables::variables.find(node.assignment_values[i - 1].id_name);
          a = Variables::variables_it->second.number_value;
        }
        else
        {
          a = node.assignment_values[i - 1].number_value;
        }
        if (node.assignment_values[i + 1].id_name.length() > 0)
        {
          Variables::variables_it = Variables::variables.find(node.assignment_values[i + 1].id_name);
          b = Variables::variables_it->second.number_value;
        }
        else
        {
          b = node.assignment_values[i + 1].number_value;
        }

        if (vals.size() > 0)
        {
          if (value.op == "+")
          {
            vals[vals.size() - 1] = vals[vals.size() - 1] + b;
          }
          else
          {
            vals[vals.size() - 1] = vals[vals.size() - 1] - b;
          }

          continue;
        }

        int val;
        if (value.op == "+")
        {
          val = a + b;
        }
        else
        {
          val = a - b;
        }

        vals.push_back(val);
      }
    }
    else if (value.op == "*")
    {
    }
  }

  return vals[0];
}

bool condition_true(Condition condition)
{
  string left_string;
  string right_string;
  string result_string;
  int left_number;
  int right_number;
  int result_number;

  vector<bool> condition_returns;

  for (int i = 0; i < condition.lefts.size(); i++)
  {
    Token left = condition.lefts[i];
    Token right = condition.rights[i];
    Token op = condition.ops[i];
    Token result;
    Token result_operator;
    bool result_exists = false;
    if (condition.results.size() > 0)
    {
      result_exists = true;
      result = condition.results[i];
      result_operator = condition.results_operators[i];
    }

    if (left.value.substr(0, 1) == "\"" && left.type == Types::lit)
    {
      left_string = left.value.substr(1, left.value.length() - 2);
    }
    else if (!is_number(left.value))
    {
      Variables::variables_it = Variables::variables.find(left.value);
      if (Variables::variables_it->second.string_value.length() > 0)
      {
        left_string = Variables::variables_it->second.string_value;
      }
      else
      {
        left_number = Variables::variables_it->second.number_value;
      }
    }
    else
    {
      left_number = std::stoi(left.value);
    }

    if (right.value.substr(0, 1) == "\"" && right.type == Types::lit)
    {
      right_string = right.value.substr(1, right.value.length() - 2);
    }
    else if (!is_number(right.value))
    {
      Variables::variables_it = Variables::variables.find(right.value);
      if (Variables::variables_it->second.string_value.length() > 0)
      {
        right_string = Variables::variables_it->second.string_value;
      }
      else
      {
        right_number = Variables::variables_it->second.number_value;
      }
    }
    else
    {
      right_number = std::stoi(right.value);
    }

    if (result_exists)
    {
      if (!is_number(result.value))
      {
        Variables::variables_it = Variables::variables.find(result.value);
        if (Variables::variables_it->second.string_value.length() > 0)
        {
          result_string = Variables::variables_it->second.string_value;
        }
        else
        {
          result_number = Variables::variables_it->second.number_value;
        }
      }
      else
      {
        if (result.value.substr(0, 1) == "\"")
        {
          result_string = result.value;
        }
        else
        {
          result_number = std::stoi(result.value);
        }
      }
    }

    if (op.value == ">")
    {
      if (left_number > right_number)
      {
        condition_returns.push_back(true);
      }
      else
      {
        condition_returns.push_back(false);
      }
    }
    else if (op.value == "<")
    {
      if (left_number < right_number)
      {
        condition_returns.push_back(true);
      }
      else
      {
        condition_returns.push_back(false);
      }
    }
    else if (op.value == "==")
    {
      if (left_string.length() > 0)
      {
        if (left_string == right_string)
        {
          condition_returns.push_back(true);
        }
        else
        {
          condition_returns.push_back(false);
        }
      }
      else if (left_number == right_number)
      {
        condition_returns.push_back(true);
      }
      else
      {
        condition_returns.push_back(false);
      }
    }
    else if (op.value == "!=")
    {
      if (left_string.length() > 0)
      {
        if (left_string != right_string)
        {
          condition_returns.push_back(true);
        }
        else
        {
          condition_returns.push_back(false);
        }
      }
      else if (left_number != right_number)
      {
        condition_returns.push_back(true);
      }
      else
      {
        condition_returns.push_back(false);
      }
    }
    else if (op.value == ">=")
    {
      if (left_number >= right_number)
      {
        condition_returns.push_back(true);
      }
      else
      {
        condition_returns.push_back(false);
      }
    }
    else if (op.value == "<=")
    {
      if (left_number <= right_number)
      {
        condition_returns.push_back(true);
      }
      else
      {
        condition_returns.push_back(false);
      }
    }
    else if (op.value == "/")
    {
      if (result_exists)
      {
        if (left_number / right_number == result_number)
        {
          condition_returns.push_back(true);
        }
        else
        {
          condition_returns.push_back(false);
        }
      }
    }
    else if (op.value == "%")
    {
      if (result_exists)
      {
        if (left_number % right_number == result_number)
        {
          condition_returns.push_back(true);
        }
        else
        {
          condition_returns.push_back(false);
        }
      }
    }
    else
    {
      condition_returns.push_back(false);
    }
  }

  if (condition.condition_seperators.size() > 0)
  {
    vector<bool> evaluate_returns;
    for (int j = 0; j < condition.condition_seperators.size(); j++)
    {
      Token seperator = condition.condition_seperators[j];
      if (seperator.value == "&&")
      {
        if (condition_returns[j] == true && condition_returns[j + 1] == true)
        {
          evaluate_returns.push_back(true);
        }
        else
        {
          evaluate_returns.push_back(false);
        }
      }
      else if (seperator.value == "||")
      {
        if (condition_returns[j] == true || condition_returns[j + 1] == true)
        {
          evaluate_returns.push_back(true);
        }
        else
        {
          evaluate_returns.push_back(false);
        }
      }
    }

    for (int i = 0; i < evaluate_returns.size(); i++)
    {
      if (!evaluate_returns[i])
      {
        return false;
      }
    }
  }
  else
  {
    if (condition_returns[0])
    {
      return true;
    }
    else
    {
      return false;
    }
  }

  return true;
}

void Interpreter::_else(vector<Node> nodes, int i, Node &parent)
{

  for (int j = i; j > 0; j--)
  {
    if (nodes[j].condition.lefts.size() > 0)
    {
      if (condition_true(nodes[j].condition))
      {
        return;
      }
    }
  }

  for (int j = 0; j < nodes[i].then.nodes.size(); j++)
  {
    if (parent.type == -1)
    {
      interpret(nodes[i].then.nodes, j, nodes[i]);
    }
    else
    {
      interpret(nodes[i].then.nodes, j, parent);
    }
  }
}

void Interpreter::else_if(vector<Node> nodes, int i, Node &parent)
{
  if (condition_true(nodes[i - 1].condition))
  {
    return;
  }
  for (int j = 0; j < nodes[i].then.nodes.size(); j++)
  {
    if (condition_true(nodes[i].condition))
    {
      if (parent.type == -1)
      {
        interpret(nodes[i].then.nodes, j, nodes[i]);
      }
      else
      {
        interpret(nodes[i].then.nodes, j, parent);
      }
    }
  }
}

void Interpreter::_if(Node node, Node &parent)
{
  Variables::If _if;
  _if.then = node.then;
  _if.condition = node.condition;
  _if.id = Variables::ifs.size();
  node.if_id = Variables::ifs.size();
  Variables::ifs.push_back(_if);

  if (condition_true(node.condition))
  {
    for (int i = 0; i < node.then.nodes.size(); i++)
    {
      interpret(node.then.nodes, i, node);
      // if (parent.type == -1)
      // {
      //   interpret(node.then.nodes, i, node);
      // }
      // else
      // {
      //   interpret(node.then.nodes, i, parent);
      // }
    }
  }
}

void Interpreter::_while(Node node, Node &parent)
{
  node.should_break = false;
  parent.should_break = false;
  while (condition_true(node.condition))
  {
    for (int i = 0; i < node.then.nodes.size(); i++)
    {
      if (node.should_break)
      {
        node.should_break = false;
        goto end_while;
      }
      if (node.should_continue)
      {
        node.should_continue = false;
        continue;
      }
      if (parent.type == -1)
      {
        interpret(node.then.nodes, i, node);
      }
      else
      {
        interpret(node.then.nodes, i, parent);
      }
    }
  }
end_while:;
};

void Interpreter::let(Node node, Node &parent)
{
  Variables::Variable var;

  if (node.variable_value_string.value.substr(0, 1) == "\"")
  {
    var.string_value = node.variable_value_string.value.substr(1, node.variable_value_string.value.length() - 2);
  }
  else
  {
    var.number_value = node.variable_value_number.value;
  }

  if (parent.type == Node_Types::function_call)
  {
    Variables::functions_it = Variables::functions.find(parent.function_name);
    if (Variables::functions_it != Variables::functions.end())
    {
      Variables::functions_it->second.variables.insert({node.variable_name, var});
    }
  }
  else if (parent.type == Node_Types::_if)
  {
    // cout << "IF" << endl;
    // cout << Variables::ifs.size() - 1 << endl;
    for (int i = 0; i < Variables::ifs.size(); i++)
    {
      if (Variables::ifs[i].id == parent.if_id)
      {
        // cout << node.variable_name << ' ' << var.number_value << endl;
        Variables::ifs[i].variables.insert({node.variable_name, var});
      }
    }
  }
  else
  {
    Variables::variables.insert({node.variable_name, var});
  }
};

int add(int a, int b)
{
  return a + b;
}

int multiply(int a, int b)
{
  return a * b;
}

void Interpreter::assign(Node node, Node &parent)
{
  if (parent.type == Node_Types::function_call)
  {
    Variables::functions_it = Variables::functions.find(parent.function_name);
    if (Variables::functions_it != Variables::functions.end())
    {
      Variables::function_variables_it = Variables::functions_it->second.variables.find(node.id_name);
      if (Variables::function_variables_it != Variables::functions_it->second.variables.end())
      {
        if (Variables::function_variables_it->second.string_value.length() > 0)
        {
          string val = evaluate_string_expression(node);
          Variables::function_variables_it->second.string_value = val;
        }
        else
        {
          int val = evaluate_expression(node);
          Variables::function_variables_it->second.number_value = val;
        }
      }
      else
      {
        std::cerr << "Could not assign new value to undefined variable: " << node.id_name << endl;
        return;
      }
    }
  }
  else
  {
    Variables::variables_it = Variables::variables.find(node.id_name);
    if (Variables::variables_it->second.string_value.length() > 0)
    {
      string val = evaluate_string_expression(node);
      Variables::variables_it->second.string_value = val;
    }
    else
    {
      int val = evaluate_expression(node);
      Variables::variables_it->second.number_value = val;
    }
  }
}

void Interpreter::_continue(vector<Node> nodes, int i, Node &parent)
{
  parent.should_continue = true;
}

void Interpreter::_break(vector<Node> nodes, int i, Node &parent)
{
  parent.should_break = true;
}

string Interpreter::_input(Node node)
{
  Node parent;
  Print::print(node, parent);
  // _print(node, parent);
  string input;
  std::cin >> input;
  return input;
}

void Interpreter::function(vector<Node> nodes, int i)
{

  Variables::Function function;

  function.parameters = nodes[i].parameters;
  function.then = nodes[i].then;

  Variables::functions.insert({nodes[i].function_call_name, function});
}

void Interpreter::call_function(vector<Node> nodes, int i)
{
  Variables::functions_it = Variables::functions.find(nodes[i].function_name);

  Node parent;
  for (int j = 0; j < Variables::functions_it->second.then.nodes.size(); j++)
  {
    interpret(Variables::functions_it->second.then.nodes, j, nodes[i]);
  }
}

void interpret(vector<Node> nodes, int i, Node &parent)
{

  Node node = nodes[i];
  switch (node.type)
  {
  case Node_Types::function_call:
    if (node.function_call_name == "print")
    {
      Print::print(node, parent);
      // _print(node, parent);
    }
    else if (node.function_call_name == "input")
    {
      string input = Interpreter::_input(node);
    }
    else
    {
      Interpreter::call_function(nodes, i);
    }

    break;
  case Node_Types::_while:
    Interpreter::_while(node, parent);
    break;
  case Node_Types::_if:
    Interpreter::_if(node, parent);
    break;
  case Node_Types::else_if:
    Interpreter::else_if(nodes, i, parent);
    break;
  case Node_Types::_else:
    Interpreter::_else(nodes, i, parent);
    break;
  case Node_Types::let:
    Interpreter::let(node, parent);
    break;
  case Node_Types::assign:
    Interpreter::assign(node, parent);
    break;
  case Node_Types::_continue:
    Interpreter::_continue(nodes, i, parent);
    break;
  case Node_Types::_break:
    Interpreter::_break(nodes, i, parent);
    break;
  case Node_Types::function:
    Interpreter::function(nodes, i);
    break;
  default:
    break;
  }
}
void run(Tree ast)
{
  Node parent;
  parent.type = -1;
  for (int i = 0; i < ast.nodes.size(); i++)
  {
    interpret(ast.nodes, i, parent);
  }
}