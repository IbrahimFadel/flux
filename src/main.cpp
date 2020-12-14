#include <iostream>
#include <fstream>
#include <istream>

#include "options.h"
#include "common.h"
#include "lexer.h"
#include "parser.h"
#include "code_generation.h"

using std::cout;
using std::endl;

int main(int argc, const char **argv)
{
  CompilerOptions options;

  std::vector<std::string> arguments(argv, argv + argc);
  int i = 0;
  for (auto &arg : arguments)
  {
    if (arg == "--optimize")
    {
      options.optimize = true;
    }
    else if (arg == "--output" || arg == "-o")
    {
      options.output_path = std::string(arguments[i + 1]);
    }
    i++;
  }

  auto file_content = get_file_content(argv[1]);
  auto tokens = tokenize(file_content);
  // print_tokens(tokens);
  auto nodes = parse_tokens(tokens);
  print_nodes(nodes);
  auto module = code_gen_nodes(std::move(nodes), options);

  module_to_obj(std::move(module));

  return 0;
}
