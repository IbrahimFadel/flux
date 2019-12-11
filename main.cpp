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

// char *get_file_input(const char *path)
// {
//   std::ifstream is(path, std::ifstream::binary);
//   if (is)
//   {
//     is.seekg(0, is.end);
//     int length = is.tellg();
//     is.seekg(0, is.beg);
//     char *buffer = new char[length];
//     is.read(buffer, length);
//     is.close();

//     return buffer;
//   }
//   return {};
// }

string get_file_input(const char *path)
{
  string content;
  char c;
  std::fstream Txt1(path, std::ios::in);

  Txt1.get(c);
  do
  {
    content += c;
    Txt1.get(c);
  } while (!Txt1.eof());
  Txt1.close();

  return content;
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
  string input = get_file_input("test.es");
  // string str(input);
  vector<Token> tokens = generate_tokens(input);

  print_tokens(tokens);

  // generate_ast(tokens);

  return 0;
}
