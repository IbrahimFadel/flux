#ifndef NODES_H
#define NODES_H

#include <string>

namespace Transformation {

class Package {
 private:
  // std::vector<unique_ptr<FnDecl>> functions;
};

class Node {
 private:
 public:
  virtual std::string toString() = 0;
};

// class ASTTransformer {
//  private:

//  public:
// };

}  // namespace Transformation

#endif