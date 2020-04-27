#include <iostream>
#include <fstream>
#include <istream>

#include "lexer.h"
#include "parser.h"

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
  std::string file_content = get_file_content("test.se");

  auto tokens = get_tokens(file_content);
  print_tokens(tokens);

  parse_tokens(tokens);

  return 0;
}