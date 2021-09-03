#include <ir/ir.h>
#include <parser/dump.h>
#include <parser/parser.h>
#include <scanner/scanner.h>
#include <token/token.h>

int main() {
  Token::init();

  std::string content = Scanner::readFile("../testData/test.pi");
  Scanner::Scanner scanner(content);
  scanner.tokenize();

  Parser::Parser parser(scanner.getTokens());
  parser.parseTokens();

  std::string astStr = Parser::astToString(parser.getAST());
  printf(astStr.c_str());

  auto mod = Codegen::generateLLVMModule(parser.getAST());
  std::string modStr = Codegen::LLVMModuleToString(mod);
  printf(modStr.c_str());

  return 0;
}