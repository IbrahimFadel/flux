#include <iostream>
#include <fstream>
#include <vector>
#include <stdlib.h>
#include <limits.h>
#include "lexer.h"
#include "parser.h"
#include "interpreter.h"

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

void print_ast(Tree ast)
{
  for (int i = 0; i < ast.nodes.size(); i++)
  {
    cout << "------ NODE ------" << endl;
    cout << ast.nodes[i] << endl;
    cout << "------ END NODE ------" << endl;
  }
}

void create_project(string name)
{
  string command = "mkdir -p " + name + "/src && touch " + name + "/{README.md,src/main.ybl} && echo '# New Yabl Project' >> " + name + "/README.md";
  system(command.c_str());
}

int main(int argc, char **argv)
{
  char path[PATH_MAX];
  std::string new_project_name;
  bool verbose = false;

  if (argc > 1)
  {
    std::vector<std::string> args(argv, argv + argc);
    for (size_t i = 1; i < args.size(); ++i)
    {
      if (args[i] == "new")
      {
        new_project_name = args[i + 1];
        create_project(new_project_name);
        break;
      }
      else if (args[i] == "-v" || args[i] == "--verbose")
      {
        verbose = true;
      }
      else
      {
        realpath(argv[i], path);
      }
    }
  }

  vector<string> input = get_file_input(path);
  vector<Token> tokens = generate_tokens(input);

  Tree ast = generate_ast(tokens);

  if (verbose)
  {
    print_tokens(tokens);
    print_ast(ast);
  }

  run(ast);

  return 0;
}
