#include <parser/dump.h>
#include <parser/parser.h>
#include <scanner/scanner.h>
#include <token/token.h>

int main() {
  Token::init();

  std::string content = Scanner::readFile("../testData/test.pi");
  Scanner::Scanner scanner(content);
  scanner.tokenize();

  // for (auto const& tok : scanner.getTokens()) {
  //   printf("%d:%s\n", tok.type, tok.value.c_str());
  // }

  Parser::Parser parser(scanner.getTokens());
  parser.parseTokens();

  Parser::ASTDump astdump(parser.getNodes());
  auto astStrRepr = astdump.toString();
  printf(astStrRepr.c_str());

  return 0;
}