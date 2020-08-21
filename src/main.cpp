#include <iostream>
#include <fstream>
#include <istream>

#include "lexer.h"
#include "parser.h"
// #include "code_generation.h"

using std::cout;
using std::endl;

std::string get_file_content(const char *path)
{
  std::ifstream in(path);
  std::string contents((std::istreambuf_iterator<char>(in)),
                       std::istreambuf_iterator<char>());
  return contents;
}

int main()
{
  std::string file_content = get_file_content("test.ss");

  auto tokens = get_tokens(file_content);
  // print_tokens(tokens);
  // run();

  auto nodes = parse_tokens(tokens);
  // print_nodes(nodes);
  // generate_llvm_ir(nodes);

  return 0;
}