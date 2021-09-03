#ifndef PARSER_H
#define PARSER_H

#include <parser/ast.h>
#include <token/token.h>

#include <iostream>
#include <map>
#include <memory>
#include <vector>

using std::unique_ptr;

namespace Parser {
class Parser {
 private:
  unique_ptr<AST> ast;
  std::vector<Token::Token> tokens;
  int curTokPtr;
  Token::Token *curTok;
  std::map<std::string, int> opPrecedence;

  template <typename... Args>
  std::string fmt(const std::string &format, Args... args);
  void error(Token::Position pos, std::string msg);
  Token::Token *eat();
  Token::Token *peek();
  Token::Token *expect(Token::TokenType type, std::string errMsg);
  Token::Token *expectRange(Token::TokenType typeBegin, Token::TokenType typeEnd, std::string errMsg);
  int getTokenPrecedence(Token::Token *tok);

  unique_ptr<Node> parseToken(Token::Token *tok);
  unique_ptr<FnDecl> parseFn();
  unique_ptr<FnReceiver> parseFnReceiver();
  unique_ptr<FnType> parseFnType();
  unique_ptr<ParamList> parseParamList();
  Param parseParam();
  unique_ptr<Expr> parseExpr();
  unique_ptr<Expr> parseBinaryExpr(int prec1);
  unique_ptr<Expr> parseUnaryExpr();
  unique_ptr<Expr> parsePostfixExpr(unique_ptr<Expr> expr);
  unique_ptr<Expr> parsePrimaryExpr();
  unique_ptr<Expr> parseCallExpr(const unique_ptr<Expr> &expr);
  unique_ptr<Expr> parseTypeExpr();
  unique_ptr<Expr> parsePrimitiveTypeExpr();
  unique_ptr<BlockStmt> parseBlockStmt();
  unique_ptr<ReturnStmt> parseReturn();
  unique_ptr<BasicLitExpr> parseBasicLit();
  unique_ptr<VarDecl> parseVarDecl(bool mut = false);
  std::vector<unique_ptr<IdentExpr>> parseIdentList();
  unique_ptr<IdentExpr> parseIdentExpr();
  unique_ptr<TypeDecl> parseTypeDecl();
  unique_ptr<InterfaceTypeExpr> parseInterfaceTypeExpr();
  unique_ptr<FieldList> parseFieldList();
  Field parseField();
  unique_ptr<StructTypeExpr> parseStructTypeExpr();
  unique_ptr<PropertyList> parsePropertyList();
  Property parseProperty();

 public:
  Parser(std::vector<Token::Token> tokens);
  void parseTokens();

  const unique_ptr<AST> &getAST() { return ast; }
};

}  // namespace Parser

#endif