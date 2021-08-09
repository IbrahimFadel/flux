#ifndef SCANNER_H
#define SCANNER_H

#include <stdio.h>

#include <fstream>
#include <iostream>
#include <memory>
#include <sstream>
#include <string>
#include <vector>

#include "../token/token.h"

namespace Scanner {

class Scanner {
 private:
  std::string src;
  std::vector<Token::Token> tokens;

  char ch;
  int offset;
  int lineOffset;
  Token::Position pos;

  template <typename... Args>
  std::string fmt(const std::string& format, Args... args);
  bool isLetter(char c);
  bool isDigit(char c);
  bool isHex(char c);
  char peek();
  void error(int offset, std::string message);

  void skipWhiteSpace();
  void next();

  std::string scanIdentifier();
  void scanDigits(int base, int& index);
  std::pair<Token::TokenType, std::string> scanNumber();
  std::string scanString();
  void scanEscape(char quote);
  std::string scanChar();
  std::string scanComment(bool singleLine);
  Token::Token scan();

 public:
  Scanner(std::string src);
  void tokenize();

  std::vector<Token::Token> getTokens() { return tokens; }
};

std::string readFile(std::string path);

}  // namespace Scanner

#endif