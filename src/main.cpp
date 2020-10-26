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

int main(int argc, const char **argv)
{
  std::string file_content;
  if (argc > 1)
  {
    file_content = get_file_content(argv[1]);
  }
  else
  {
    file_content = get_file_content("test.ss");
  }

  auto tokens = get_tokens(file_content);
   print_tokens(tokens);

  auto nodes = parse_tokens(tokens);
  code_gen(std::move(nodes));

  return 0;
}
