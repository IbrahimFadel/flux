#include <gtest/gtest.h>
#include <scanner/scanner.h>
#include <token/token.h>

struct TokenTest {
  Token::TokenType type;
  std::string value;
};

//TODO: add more tests
static const TokenTest tokens[] = {
    {Token::IDENT, "hello"},
    {Token::IDENT, "_hello"},
    {Token::IDENT, "hello_"},
    {Token::IDENT, "_hello_"},

    {Token::INT, "1"},
    {Token::INT, "1_000"},

    {Token::INT, "0x1"},
    {Token::INT, "0xF"},
    {Token::INT, "0xf"},
    {Token::INT, "0xFF_AC"},
    {Token::INT, "0xFf_Ab"},

    {Token::INT, "0b10"},
    {Token::INT, "0b0"},

    {Token::FLOAT, "1.0"},
    {Token::FLOAT, "1_000.0"},
    {Token::FLOAT, "9201.193_10"},
    {Token::FLOAT, "9_201.193_10"},

    {Token::LPAREN, "("},
    {Token::RPAREN, ")"},
    {Token::LBRACE, "{"},
    {Token::RBRACE, "}"},
    {Token::ARROW, "->"},

    {Token::FN, "fn"},
    {Token::RETURN, "return"},
};

TEST(Scanner, ScansAllTokens) {
  Token::init();  // initializes keywords

  std::string src;

  for (auto const &tok : tokens) {
    src += tok.value;
    src += ' ';
  }

  Scanner::Scanner scanner(src);
  scanner.tokenize();

  auto toks = scanner.getTokens();
  for (int i = 0; i < toks.size(); i++) {
    auto tok = toks[i];
    EXPECT_EQ(tok.type, tokens[i].type);
    EXPECT_EQ(tok.value, tokens[i].value);
  }
}
