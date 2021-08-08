#include "parser.h"

using namespace Parser;

Parser::Parser::Parser(std::vector<Token::Token> tokens) : tokens(tokens) {
  curTokPtr = 0;
  curTok = &tokens[0];
  opPrecedence = {
      {"=", 2},
      {"&&", 3},
      {"||", 5},
      {"<", 10},
      {">", 10},
      {"<=", 10},
      {">=", 10},
      {"==", 10},
      {"!=", 10},
      {"+", 20},
      {"-", 20},
      {"*", 40},
      {"/", 40},
      {".", 50},
      {"->", 50},
  };
}

// https://stackoverflow.com/questions/2342162/stdstring-formatting-like-sprintf
template <typename... Args>
std::string Parser::Parser::fmt(const std::string &format, Args... args) {
  int size_s = std::snprintf(nullptr, 0, format.c_str(), args...) + 1;  // Extra space for '\0'
  if (size_s <= 0) {
    throw std::runtime_error("Error during formatting.");
  }
  auto size = static_cast<size_t>(size_s);
  auto buf = std::make_unique<char[]>(size);
  std::snprintf(buf.get(), size, format.c_str(), args...);
  return std::string(buf.get(), buf.get() + size - 1);  // We don't want the '\0' inside
}

void Parser::Parser::parseTokens() {
  for (const auto &tok : tokens) {
    parseToken(tok);
  }
}

void Parser::Parser::error(Token::Position pos, std::string msg) {
  printf("parser error:\n%d:%d\t%s\n", pos.line, pos.col, msg.c_str());
  exit(1);
}

//TODO: check if it's at end of tokens
void Parser::Parser::eat() {
  curTokPtr++;
  curTok = &tokens[curTokPtr];
}

//TODO: check if it's at end of tokens
Token::Token *Parser::Parser::peek() {
  return &tokens[curTokPtr + 1];
}

Token::Token *Parser::Parser::expect(Token::TokenType type, std::string errMsg) {
  if (curTok->type != type) error(curTok->pos, errMsg);
  auto tok = curTok;
  eat();
  return tok;
}

Token::Token *Parser::Parser::expectRange(Token::TokenType typeBegin, Token::TokenType typeEnd, std::string errMsg) {
  if (curTok->type <= typeBegin || curTok->type >= typeEnd) error(curTok->pos, errMsg);
  auto tok = curTok;
  eat();
  return tok;
}

int Parser::Parser::getTokenPrecedence(Token::Token *tok) {
  if (!opPrecedence.contains(tok->value)) return -1;
  return opPrecedence[tok->value];
}

unique_ptr<Node> Parser::Parser::parseToken(Token::Token tok) {
  switch (tok.type) {
    case Token::FN:
      return parseFn();
    case Token::COMMENT:
      eat();
      return nullptr;
    default:
      error(curTok->pos, fmt("could not parse token: %s", curTok->value.c_str()));
      break;
  }
  return nullptr;
}

unique_ptr<FnDecl> Parser::Parser::parseFn() {
  expect(Token::FN, "expected 'fn'");
  expect(Token::IDENT, "expected identifier following 'fn'");
  expect(Token::LPAREN, "expected '(' following function name");

  auto paramList = parseParamList();

  expect(Token::RPAREN, "expected ')' following param list");  // should already be handled in parseParamList, but for clarity, we have it here

  unique_ptr<Expr> retTy;
  if (curTok->type != Token::ARROW) {
    retTy = std::make_unique<PrimitiveTypeExpr>(Token::VOID);
  } else {
    eat();
    retTy = parseTypeExpr();
  }

  expect(Token::LBRACE, "expected '{' at beginning of function body");

  auto body = parseBlockStmt();

  expect(Token::RBRACE, "expected '}' at end of function body");

  return nullptr;
}

unique_ptr<BlockStmt> Parser::Parser::parseBlockStmt() {
  std::vector<unique_ptr<Node>> nodes;
  while (curTok->type != Token::RBRACE) {
    auto node = parseToken(*curTok);
    nodes.push_back(std::move(node));
  }

  return std::make_unique<BlockStmt>(std::move(nodes));
}

unique_ptr<ParamList> Parser::Parser::parseParamList() {
  std::vector<Param> params;
  while (curTok->type != Token::RPAREN) {
    auto param = parseParam();
    if (curTok->type != Token::COMMA) {
      if (curTok->type != Token::RPAREN) error(curTok->pos, "expected ')' at end of param list");
    } else {
      eat();
    }
  }
  return std::make_unique<ParamList>(std::move(params));
}

Param Parser::Parser::parseParam() {
  Param param;
  param.mut = false;
  if (curTok->type == Token::MUT) {
    eat();
    param.mut = true;
  }
  param.type = parseTypeExpr();
  param.name = expect(Token::IDENT, "expected identifier in function param")->value;
  return param;
}

unique_ptr<Expr> Parser::Parser::parseTypeExpr() {
  if (curTok->type > Token::types_begin && curTok->type < Token::types_end) {
    return parsePrimitiveTypeExpr();
  } else {
    error(curTok->pos, "unimplemented type");
    return nullptr;
  }
}

unique_ptr<Expr> Parser::Parser::parsePrimitiveTypeExpr() {
  Token::TokenType ty = expectRange(Token::types_begin, Token::types_end, "expected a type in primitive type expression")->type;
  if (curTok->type != Token::ASTERISK) return std::make_unique<PrimitiveTypeExpr>(ty);

  auto ptrTy = std::make_unique<PointerType>(std::make_unique<PrimitiveTypeExpr>(ty));
  eat();
  while (curTok->type == Token::ASTERISK) {
    //TODO: does this cause issues cus c++ sucks?
    ptrTy->setPointerToType(std::move(ptrTy));
    eat();
  }

  return ptrTy;
}

unique_ptr<Expr> Parser::Parser::parseExpr() {
  return parseBinaryExpr(1);
}

unique_ptr<Expr> Parser::Parser::parseBinaryExpr(int prec1) {
  auto x = parseUnaryExpr();
  while (true) {
    int oprec = getTokenPrecedence(curTok);
    Token::TokenType op = curTok->type;

    if (oprec < prec1) {
      return x;
    }

    eat();
    auto y = parseBinaryExpr(oprec + 1);

    x = std::make_unique<BinaryExpr>(std::move(x), op, std::move(y));
    x = parsePostfixExpr(std::move(x));
  }
}

unique_ptr<Expr> Parser::Parser::parseUnaryExpr() {
  switch (curTok->type) {
    case Token::AMPERSAND:
    case Token::ASTERISK:
      error(curTok->pos, "unimplemented unary expression");
      return nullptr;
    default:
      return parsePrimaryExpr();
      break;
  }
}

unique_ptr<Expr> Parser::Parser::parsePostfixExpr(unique_ptr<Expr> expr) {
  switch (curTok->type) {
    case Token::LPAREN:
      return parseCallExpr(expr);
    default:
      return expr;
  }
}

unique_ptr<Expr> Parser::Parser::parsePrimaryExpr() {
  return nullptr;
}

unique_ptr<Expr> Parser::Parser::parseCallExpr(const unique_ptr<Expr> &expr) {
  return nullptr;
}