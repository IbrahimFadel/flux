#include <gtest/gtest.h>
#include <parser/parser.h>
#include <scanner/scanner.h>

static const std::string testFilesDir = "../testData/parserTestFiles/";

TEST(Parser, FunctionDeclaration) {
  EXPECT_EQ(0, 0);
  // std::vector<unique_ptr<Parser::Node>> funcBodyExpectedNodes = {};
  // auto body = std::make_unique<Parser::BlockStmt>(std::move(funcBodyExpectedNodes));
  // auto fnDecl = std::make_unique<Parser::FnDecl>(
  //     std::make_unique<Parser::FnReceiver>(),
  //     "main",
  //     std::make_unique<Parser::FnType>(),
  //     std::move(body));
  // std::vector<unique_ptr<Parser::Node>> funcExpectedNodes;
  // funcExpectedNodes.push_back(std::move(fnDecl));

  // std::string content = Scanner::readFile("../testData/parserTestFiles/func.pi");
  // Scanner::Scanner scanner(content);
  // scanner.tokenize();

  // for (int i = 0; i < scanner.getTokens().size(); i++) {
  //   EXPECT_EQ(2, 1);
  // }

  // Parser::Parser parser(scanner.getTokens());
  // for (int i = 0; i < parser.getNodes().size(); i++) {
  //   EXPECT_EQ(0, 1);
  // switch (node)
  // {
  // case /* constant-expression */:
  /* code */
  // break;

  // default:
  // break;
  // }
  // EXPECT_EQ(parser.getNodes()[i], funcExpectedNodes[i]);
}