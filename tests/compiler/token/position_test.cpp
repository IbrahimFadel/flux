#include <gtest/gtest.h>
#include <token/position.h>

static const char *MOCK_FILENAME = "test.pi";
static const int MOCK_LINE = 3;
static const int MOCK_INVALID_LINE = 0;
static const int MOCK_COL = 10;
static const int MOCK_INVALID_COL = 0;

TEST(TokenPosition, FilenameAndInvalidPosition) {
  Token::Position pos;
  pos.filename = MOCK_FILENAME;
  pos.line = MOCK_INVALID_LINE;
  pos.col = MOCK_INVALID_COL;
  std::string result = pos.toString();

  EXPECT_STREQ(result.c_str(), MOCK_FILENAME);
}

TEST(TokenPosition, FilenameAndValidPosition) {
  Token::Position pos;
  pos.filename = MOCK_FILENAME;
  pos.line = MOCK_LINE;
  pos.col = MOCK_COL;
  std::string result = pos.toString();

  std::string expected = MOCK_FILENAME + std::string(":") + std::to_string(MOCK_LINE) + std::string(":") + std::to_string(MOCK_COL);
  EXPECT_STREQ(result.c_str(), expected.c_str());
}

TEST(TokenPosition, NoFilenameAndInvalidPosition) {
  Token::Position pos;
  pos.filename = "";
  pos.line = MOCK_INVALID_LINE;
  pos.col = MOCK_INVALID_COL;
  std::string result = pos.toString();

  EXPECT_STREQ(result.c_str(), "-");
}

TEST(TokenPosition, NoFilenameAndValidPosition) {
  Token::Position pos;
  pos.filename = "";
  pos.line = MOCK_LINE;
  pos.col = MOCK_COL;
  std::string result = pos.toString();

  std::string expected = std::to_string(MOCK_LINE) + ":" + std::to_string(MOCK_COL);
  EXPECT_STREQ(result.c_str(), expected.c_str());
}
