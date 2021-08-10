#ifndef AST_H
#define AST_H

#include <token/token.h>

#include <memory>
#include <string>
#include <vector>

using std::unique_ptr;

namespace Parser {

class IdentExpr;
class BlockStmt;

class Node {
 public:
  virtual std::string toString() = 0;
};

////////////////////////////////////

class Expr : public Node {
 public:
  virtual std::string toString() = 0;
};

class Stmt : public Node {
 public:
  virtual std::string toString() = 0;
};

class Decl : public Node {
 public:
  virtual std::string toString() = 0;
};

class Comment : public Node {
 private:
  std::string text;

 public:
  virtual std::string toString() = 0;
};

////////////////////////////////////

class FnReceiver : public Expr {
 private:
  std::unique_ptr<Expr> type;
  std::unique_ptr<IdentExpr> name;

 public:
  FnReceiver(std::unique_ptr<Expr> type, std::unique_ptr<IdentExpr> name) : type(std::move(type)), name(std::move(name)){};

  std::string toString();
};

struct Param {
  bool mut;
  unique_ptr<Expr> type;
  std::string name;
};

class ParamList : public Expr {
 private:
  std::vector<Param> params;

 public:
  ParamList(std::vector<Param> params) : params(std::move(params)){};
  std::string toString();
};

class FnType : public Expr {
 private:
  unique_ptr<ParamList> paramList;
  unique_ptr<Expr> returnType;

 public:
  FnType(unique_ptr<ParamList> paramList, unique_ptr<Expr> returnType) : paramList(std::move(paramList)), returnType(std::move(returnType)){};
  std::string toString();
};

class FnDecl : public Decl {
 private:
  unique_ptr<FnReceiver> receiver;
  std::string name;
  unique_ptr<FnType> type;
  unique_ptr<BlockStmt> body;

 public:
  FnDecl(unique_ptr<FnReceiver> receiver, std::string name, unique_ptr<FnType> type, unique_ptr<BlockStmt> body) : receiver(std::move(receiver)), name(name), type(std::move(type)), body(std::move(body)){};
  std::string toString();
};

class VarDecl : public Decl {
 private:
  bool mut;
  unique_ptr<Expr> type;
  std::vector<unique_ptr<IdentExpr>> names;
  std::vector<unique_ptr<Expr>> values;

 public:
  VarDecl(bool mut, unique_ptr<Expr> type, std::vector<unique_ptr<IdentExpr>> names, std::vector<unique_ptr<Expr>> values) : mut(mut), type(std::move(type)), names(std::move(names)), values(std::move(values)){};
  std::string toString();
};

class TypeDecl : public Decl {
 private:
  std::string name;
  unique_ptr<Expr> type;

 public:
  TypeDecl(std::string name, unique_ptr<Expr> type) : name(name), type(std::move(type)){};
  std::string toString();
};

////////////////////////////////////

class BinaryExpr : public Expr {
 private:
  unique_ptr<Expr> x;
  Token::TokenType op;
  unique_ptr<Expr> y;

 public:
  BinaryExpr(unique_ptr<Expr> x, Token::TokenType op, unique_ptr<Expr> y) : x(std::move(x)), op(op), y(std::move(y)){};
  std::string toString();
};

class PrimitiveTypeExpr : public Expr {
 private:
  Token::TokenType type;

 public:
  PrimitiveTypeExpr(Token::TokenType type) : type(type){};
  std::string toString();
};

class PointerTypeExpr : public Expr {
 private:
  unique_ptr<Expr> pointerToType;

 public:
  PointerTypeExpr(unique_ptr<Expr> pointerToType) : pointerToType(std::move(pointerToType)){};
  std::string toString();

  void setPointerToType(unique_ptr<Expr> ty) { pointerToType = std::move(ty); };
};

class BasicLitExpr : public Expr {
 private:
  Token::TokenType type;
  std::string value;

 public:
  BasicLitExpr(Token::TokenType type, std::string value) : type(type), value(value){};
  std::string toString();
};

class NullExpr : public Expr {
 private:
 public:
  NullExpr(){};
  std::string toString();
};

class IdentExpr : public Expr {
 private:
  std::string value;

 public:
  IdentExpr(std::string value) : value(value){};
  std::string toString();
};

struct Field {
  std::string name;
  unique_ptr<ParamList> params;
  unique_ptr<Expr> returnType;
};

class FieldList : public Expr {
 private:
  std::vector<Field> fields;

 public:
  FieldList(std::vector<Field> fields) : fields(std::move(fields)){};
  std::string toString();
};

class InterfaceTypeExpr : public Expr {
 private:
  unique_ptr<FieldList> methods;

 public:
  InterfaceTypeExpr(unique_ptr<FieldList> methods) : methods(std::move(methods)){};
  std::string toString();
};

struct Property {
  bool pub;
  bool mut;
  unique_ptr<Expr> type;
  std::vector<unique_ptr<IdentExpr>> names;
};

class PropertyList : public Expr {
 private:
  std::vector<Property> properties;

 public:
  PropertyList(std::vector<Property> fields) : properties(std::move(fields)){};
  std::string toString();
};

class StructTypeExpr : public Expr {
 private:
  unique_ptr<PropertyList> properties;

 public:
  StructTypeExpr(unique_ptr<PropertyList> properties) : properties(std::move(properties)){};
  std::string toString();
};

////////////////////////////////////

class BlockStmt : public Stmt {
  std::vector<unique_ptr<Node>> list;

 public:
  BlockStmt(std::vector<unique_ptr<Node>> list) : list(std::move(list)){};
  std::string toString();
};

class ReturnStmt : public Stmt {
 private:
  unique_ptr<Expr> expr;

 public:
  ReturnStmt(unique_ptr<Expr> expr) : expr(std::move(expr)){};
  std::string toString();
};

//////////////////////////////////

}  // namespace Parser

#endif