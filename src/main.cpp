#include <iostream>
#include <fstream>
#include <istream>

#include "lexer.h"
#include "parser.h"
#include "code_generation.h"

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
  // std::cout << "ss >>> ";
  // std::string in;
  // std::getline(std::cin, in);

  // auto tokens = get_tokens(in);
  // auto nodes = parse_tokens(tokens);
  // generate_llvm_ir(nodes);
  std::string file_content = get_file_content("test.se");

  auto tokens = get_tokens(file_content);
  print_tokens(tokens);

  std::vector<Node *> nodes = parse_tokens(tokens);
  print_nodes(nodes);
  generate_llvm_ir(nodes);

  return 0;
}