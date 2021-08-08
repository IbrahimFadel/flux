#include <parser/parser.h>
#include <scanner/scanner.h>
#include <token/token.h>

#include <fstream>
#include <streambuf>
#include <string>

std::string readFile(std::string path) {
  std::ifstream t(path);
  std::string str;

  t.seekg(0, std::ios::end);
  str.reserve(t.tellg());
  t.seekg(0, std::ios::beg);

  str.assign((std::istreambuf_iterator<char>(t)),
             std::istreambuf_iterator<char>());
  return str;
}

int main() {
  Token::init();

  std::string content = readFile("../testData/test.pi");
  Scanner::Scanner scanner(content);
  scanner.tokenize();

  Parser::Parser parser(scanner.getTokens());
  parser.parseTokens();

  return 0;
}