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

struct foo
{
  int test = 0;
};

struct test
{
  std::variant<int, foo> integer, my_foo;
};

int main()
{
  std::string file_content = get_file_content("test.se");

  auto tokens = get_tokens(file_content);
  print_tokens(tokens);

  std::vector<Node *> nodes = parse_tokens(tokens);
  print_nodes(nodes);
  generate_llvm_ir(nodes);

  // foo my_foo;
  // test my_test;
  // my_test.my_foo = my_foo;

  // cout << std::get<foo>(my_test.my_foo).test << endl;

  return 0;
}