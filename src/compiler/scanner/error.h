#ifndef ERROR_H
#define ERROR_H

#include <token/position.h>

#include <utility>

namespace Scanner {

struct Error {
  Token::Position pos;
  std::string msg;

  std::string toString();
};

std::string Error::toString() {
  if (pos.filename != "" || pos.isValid()) {
    return pos.toString() + ": " + msg;
  }
  return msg;
}

}  // namespace Scanner

#endif