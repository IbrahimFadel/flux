#include <parser/dump.h>
#include <parser/parser.h>
#include <scanner/scanner.h>
#include <token/token.h>

int main() {
  Token::init();

  std::string content = Scanner::readFile("../testData/test.pi");
  Scanner::Scanner scanner(content);
  scanner.tokenize();

  // for (auto const &tok : scanner.getTokens()) {
  //   auto x = tok.pos;
  //   auto y = tok.type;
  //   auto z = tok.value;
  // }
  // while (curTok->type != Token::_EOF) {
  // auto node = parseToken(curTok);
  // nodes.push_back(std::move(node));
  // }

  // std::cout << "HI" << '\n';

  // auto test = scanner.getTokens()[0].pos;

  // for (auto const& tok : scanner.getTokens()) {
  // printf("%d:%s\n", tok.type, tok.value.c_str());
  // }

  Parser::Parser parser(scanner.getTokens());
  parser.parseTokens();

  // Parser::ASTDump astdump(parser.getNodes());
  // auto astStrRepr = astdump.toString();
  // printf(astStrRepr.c_str());

  return 0;
}