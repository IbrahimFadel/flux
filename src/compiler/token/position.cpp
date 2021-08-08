#include "position.h"

bool Token::Position::isValid() { return line > 0 && col > 0; }

std::string Token::Position::toString() {
  std::string s = filename;
  if (isValid()) {
    if (s != "") {
      s += ":";
    }
    s += std::to_string(line);
    if (col != 0) {
      s += ":" + std::to_string(col);
    }
  }
  if (s == "") {
    s = "-";
  }
  return s;
}