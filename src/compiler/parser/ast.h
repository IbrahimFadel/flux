#ifndef AST_H
#define AST_H

#include <token/token.h>

#include <memory>
#include <string>
#include <vector>

using std::unique_ptr;

namespace Parser {

class BlockStmt;

class Node {
};

////////////////////////////////////

class Expr : public Node {
};

class Stmt : public Node {
};

class Decl : public Node {
};

class Comment : public Node {
  std::string text;
};

////////////////////////////////////

class FnReceiver {
};

class FnType : public Expr {
};

struct Param {
  bool mut;
  unique_ptr<Expr> type;
  std::string name;
};

class ParamList {
 private:
  std::vector<Param> params;

 public:
  ParamList(std::vector<Param> params) : params(std::move(params)){};
};

class FnDecl : public Decl {
  FnReceiver receiver;
  std::string name;
  FnType type;
  unique_ptr<BlockStmt> body;
};

////////////////////////////////////

class BinaryExpr : public Expr {
 private:
  unique_ptr<Expr> x;
  Token::TokenType op;
  unique_ptr<Expr> y;

 public:
  BinaryExpr(unique_ptr<Expr> x, Token::TokenType op, unique_ptr<Expr> y) : x(std::move(x)), op(op), y(std::move(y)){};
};

class PrimitiveTypeExpr : public Expr {
 private:
  Token::TokenType type;

 public:
  PrimitiveTypeExpr(Token::TokenType type) : type(type){};
};

class PointerType : public Expr {
 private:
  unique_ptr<Expr> pointerToType;

 public:
  PointerType(unique_ptr<Expr> pointerToType) : pointerToType(std::move(pointerToType)){};

  void setPointerToType(unique_ptr<Expr> ty) { pointerToType = std::move(ty); };
};

////////////////////////////////////

class BlockStmt : public Stmt {
  std::vector<unique_ptr<Node>> list;

 public:
  BlockStmt(std::vector<unique_ptr<Node>> list) : list(std::move(list)){};
};

}  // namespace Parser

#endif