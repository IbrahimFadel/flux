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
  std::vector<std::string> arguments(argv, argv + argc);
  int i = 0;
  for (auto &arg : arguments)
  {
    if (arg == "--optimize")
    {
      optimize = true;
    }
  }

  auto file_content = get_file_content(argv[1]);
  auto tokens = tokenize(file_content);
  // print_tokens(tokens);
  auto nodes = parse_tokens(tokens);
  // print_nodes(nodes);
  auto module = code_gen_nodes(std::move(nodes));

  return 0;
}
