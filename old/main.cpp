#include <iostream>
#include <fstream>
#include <vector>
#include <ctype.h>

#include "lexer.h"
#include "parser.h"

using namespace Lexer;

void print_tokens(std::vector<Token> tokens)
{
  for (int i = 0; i < tokens.size(); i++)
  {
    Token tok = tokens[i];
    std::cout << "[ " << tok.type << ": "
              << "\"" << tok.value << "\" ]" << std::endl;
  }
}

int main()
{
  char *input = get_file_input("test.es");
  std::string str(input);
  std::vector<Token> tokens = generate_tokens(input);

  // print_tokens(tokens);

  generate_ast(tokens);

  return 0;
}