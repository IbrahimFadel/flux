#include <iostream>
#include <fstream>
#include <istream>

#include "common.h"
#include "lexer.h"
#include "parser.h"
#include "code_generation.h"

using std::cout;
using std::endl;

int main(int argc, const char **argv)
{
  auto file_content = get_file_content(argv[1]);
  auto tokens = tokenize(file_content);
  // print_tokens(tokens);
  auto nodes = parse_tokens(tokens);
  // print_nodes(nodes);
  auto module = code_gen_nodes(std::move(nodes));

  return 0;
}
