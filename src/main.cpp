#include <iostream>
#include <fstream>
#include <vector>
#include <stdlib.h>
#include <limits.h>
#include "lexer.h"
#include "parser.h"

using namespace Lexer;
using namespace Parser;

using std::cout;
using std::endl;
using std::string;
using std::vector;

vector<string> get_file_input(const char *path)
{
  vector<string> content;
  std::ifstream input(path);
  for (std::string line; getline(input, line);)
  {
    content.push_back(line);
  }
  return content;
}

void print_tokens(vector<Token> tokens)
{
  for (int i = 0; i < tokens.size(); i++)
  {
    cout << "[ " << tokens[i].type << ":"
         << " '" << tokens[i].value << "' ]"
         << " - ln:" << tokens[i].line_number << " pos:" << tokens[i].line_position << endl;
  }
}

void print_node(Node node, vector<Position> &end_positions)
{
  if (node.type != -1)
  {
    if (node.then.nodes.size() > 0)
    {
      for (int i = 0; i < node.then.nodes.size(); i++)
      {
        print_node(node.then.nodes[i], end_positions);
      }
    }
    else
    {
      // ! Store parent in node, so that we can check if it's inside the while loop or if statement, to print -- END THEN --
      // ? If node.parent.end.line_number == end_positions[i].line_number
      for (int i = 0; i < end_positions.size(); i++)
      {
        cout << end_positions[i].line_number << ' ' << end_positions[i].line_position << endl;
        cout << node.parent->then.end.line_number << ' ' << node.parent->then.end.line_position << endl;
        // cout << end_positions[i
        // cout << end_positions[i].line_number << ' ' << node.then.end.line_number << endl;
        // Node * parent_node->node.parent;
        // if (end_positions[i].line_number == node.parent.end.line_number && end_positions[i].line_position == node.parent.end.line_position)
        // {
        // cout << "------ END THEN ------" << endl;
        // }
        // else if(end_positions[i].line_number == node. && end_positions[i].line_position == node.then.end.line_position)
      }
      if (node.type == Node_Types::_if)
      {
        cout << "------ IF ------" << endl;
        cout << endl
             << "------ CONDITION ------" << endl;
        cout << node.condition.left.value << ' ' << node.condition.op.value << ' ' << node.condition.right.value << endl;
        cout << "------ THEN ------" << endl;
        end_positions.push_back(node.then.end);
      }
      if (node.type == Node_Types::print)
      {
        cout << "PRINT - " << node.print_value << endl;
      }
      // cout << " => " << node.type << endl;
    }
  }
}

void print_ast(Tree ast)
{
  // cout << ast.nodes[0] << endl;

  for (int i = 0; i < ast.nodes.size(); i++)
  {
    cout << "------ NODE ------" << endl;
    cout << ast.nodes[i] << endl;
    cout << "------ END NODE ------" << endl;
  }
  //   vector<Position> end_positions;
  //   cout << "------ START AST ------" << endl;
  //   cout << endl;

  //   for (int i = 0; i < ast.nodes.size(); i++)
  //   {
  //     Node node = ast.nodes[i];
  //     print_node(node, end_positions);
  //   }

  //   cout << endl;
  //   cout << "------ END AST ------" << endl;
}

int main(int argc, char **argv)
{
  char path[PATH_MAX];

  if (argc > 1)
  {
    realpath(argv[1], path);
  }
  else
  {
    std::cerr << "Please supply input file" << endl;
    return 0;
  }

  vector<string> input = get_file_input(path);
  vector<Token> tokens = generate_tokens(input);

  // print_tokens(tokens);

  Tree ast = generate_ast(tokens);
  print_ast(ast);

  return 0;
}
