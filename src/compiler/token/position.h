#ifndef POSITION_H
#define POSITION_H

#include <string>

namespace Token {

struct Position {
  std::string filename = "";
  int line = 0;
  int col = 0;

  bool isValid();
  std::string toString();
};

}  // namespace Token

#endif