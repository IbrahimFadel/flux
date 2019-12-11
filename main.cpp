#include <iostream>
#include <fstream>
#include <vector>
#include "lexer.h"
#include "parser.h"

using namespace Lexer;

using std::cout;
using std::endl;
using std::string;
using std::vector;

char *get_file_input(const char *path)
{
  std::ifstream is(path, std::ifstream::binary);
  if (is)
  {
    is.seekg(0, is.end);
    int length = is.tellg();
    is.seekg(0, is.beg);
    char *buffer = new char[length];
    is.read(buffer, length);
    is.close();

    return buffer;
  }
  return {};
}

void print_tokens(vector<Token> tokens)
{
  for (int i = 0; i < tokens.size(); i++)
  {
    cout << "[ " << tokens[i].type << ":"
         << " '" << tokens[i].value << "' ]" << endl;
  }
}

int main()
{
  char *input = get_file_input("test.yl");
  string str(input);
  vector<Token> tokens = generate_tokens(input);

  print_tokens(tokens);

  // generate_ast(tokens);

  return 0;
}
