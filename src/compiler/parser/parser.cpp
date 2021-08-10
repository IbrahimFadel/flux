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
  auto cur = &tokens[0];
  auto x = cur->pos;
  auto y = cur->type;
  auto z = cur->value;

  auto a = curTok->pos;
  // auto b = curTok->type;
  // auto c = curTok->value;
  // for (auto const &tok : tokens) {
  //   auto x = tok.pos;
  //   auto y = tok.type;
  //   auto z = tok.value;
  // }
  // auto type = curTok->type;
  // auto t = curTok->value;
  // auto x = curTok->pos;
  // while (curTok->type != Token::_EOF) {
  //   auto node = parseToken(curTok);
  //   nodes.push_back(std::move(node));
  // }
}

void Parser::Parser::error(Token::Position pos, std::string msg) {
  printf("parser error:\n%d:%d\t%s\n", pos.line, pos.col, msg.c_str());
  exit(1);
}

//TODO: check if it's at end of tokens
Token::Token *Parser::Parser::eat() {
  curTokPtr++;
  curTok = &tokens[curTokPtr];
  return curTok;
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

unique_ptr<Node> Parser::Parser::parseToken(Token::Token *tok) {
  if (tok->type < Token::types_end && tok->type > Token::types_begin)
    return parseVarDecl();
  else if (tok->type == Token::MUT) {
    eat();
    return parseVarDecl(true);
  } else if (tok->type == Token::FN)
    return parseFn();
  else if (tok->type == Token::RETURN)
    return parseReturn();
  else if (tok->type == Token::TYPE)
    return parseTypeDecl();
  else if (tok->type == Token::COMMENT) {
    eat();
    return nullptr;
  } else {
    error(curTok->pos, fmt("could not parse token: %s", curTok->value.c_str()));
  }
  return nullptr;
}

unique_ptr<TypeDecl> Parser::Parser::parseTypeDecl() {
  expect(Token::TYPE, "expected 'type' in type declaration");
  std::string name = expect(Token::IDENT, "expected identifier in type declaration")->value;
  auto type = parseTypeExpr();
  return std::make_unique<TypeDecl>(name, std::move(type));
}

unique_ptr<VarDecl> Parser::Parser::parseVarDecl(bool mut) {
  auto type = parseTypeExpr();
  auto names = parseIdentList();
  std::vector<unique_ptr<Expr>> values;
  if (curTok->type != Token::EQ) {
    for (int i = 0; i < names.size(); i++) {
      values.push_back(std::make_unique<NullExpr>());
    }
  } else {
    eat();
    for (int i = 0; i < names.size(); i++) {
      auto val = parseExpr();
      values.push_back(std::move(val));
      if (curTok->type == Token::COMMA)
        eat();
    }
  }
  return std::make_unique<VarDecl>(mut, std::move(type), std::move(names), std::move(values));
}

std::vector<unique_ptr<IdentExpr>> Parser::Parser::parseIdentList() {
  std::vector<unique_ptr<IdentExpr>> idents;

  idents.push_back(parseIdentExpr());
  while (curTok->type == Token::COMMA) {
    eat();
    idents.push_back(parseIdentExpr());
  }

  return idents;
}

unique_ptr<IdentExpr> Parser::Parser::parseIdentExpr() {
  std::string value = expect(Token::IDENT, "expected identifier")->value;
  return std::make_unique<IdentExpr>(value);
}

unique_ptr<ReturnStmt> Parser::Parser::parseReturn() {
  expect(Token::RETURN, "expected 'return' statement");
  auto expr = parseExpr();
  return std::make_unique<ReturnStmt>(std::move(expr));
}

unique_ptr<FnDecl> Parser::Parser::parseFn() {
  expect(Token::FN, "expected 'fn'");

  unique_ptr<FnReceiver> receiver = nullptr;
  if (curTok->type == Token::LPAREN) {
    receiver = parseFnReceiver();
  }

  std::string name = expect(Token::IDENT, "expected identifier following 'fn'")->value;

  auto fnType = parseFnType();

  expect(Token::LBRACE, "expected '{' at beginning of function body");

  auto body = parseBlockStmt();

  expect(Token::RBRACE, "expected '}' at end of function body");

  return std::make_unique<FnDecl>(std::move(receiver), name, std::move(fnType), std::move(body));
}

unique_ptr<FnReceiver> Parser::Parser::parseFnReceiver() {
  expect(Token::LPAREN, "expected '(' in function receiver");

  auto type = parseTypeExpr();
  auto name = parseIdentExpr();

  expect(Token::RPAREN, "expected ')' at end of function receiver");

  return std::make_unique<FnReceiver>(std::move(type), std::move(name));
}

unique_ptr<FnType> Parser::Parser::parseFnType() {
  expect(Token::LPAREN, "expected '(' following function name");

  auto paramList = parseParamList();

  expect(Token::RPAREN, "expected ')' following param list");  // should already be handled in parseParamList, but for clarity, we have it here

  unique_ptr<Expr> retType;
  if (curTok->type != Token::ARROW) {
    retType = std::make_unique<PrimitiveTypeExpr>(Token::VOID);
  } else {
    eat();
    retType = parseTypeExpr();
  }

  return std::make_unique<FnType>(std::move(paramList), std::move(retType));
}

unique_ptr<BlockStmt> Parser::Parser::parseBlockStmt() {
  std::vector<unique_ptr<Node>> nodes;
  while (curTok->type != Token::RBRACE) {
    auto node = parseToken(curTok);
    nodes.push_back(std::move(node));
    if (curTok->type == Token::_EOF) {
      error(curTok->pos, "expected '}' at end of block statement");
    }
  }

  return std::make_unique<BlockStmt>(std::move(nodes));
}

unique_ptr<ParamList> Parser::Parser::parseParamList() {
  std::vector<Param> params;
  while (curTok->type != Token::RPAREN) {
    auto param = parseParam();
    params.push_back(std::move(param));
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
  if (curTok->type > Token::types_begin && curTok->type < Token::types_end)
    return parsePrimitiveTypeExpr();
  else if (curTok->type == Token::IDENT)
    return parseIdentExpr();
  else if (curTok->type == Token::INTERFACE)
    return parseInterfaceTypeExpr();
  else if (curTok->type == Token::STRUCT)
    return parseStructTypeExpr();
  else {
    error(curTok->pos, fmt("unimplemented type: %s", curTok->value.c_str()));
    return nullptr;
  }
}

unique_ptr<StructTypeExpr> Parser::Parser::parseStructTypeExpr() {
  expect(Token::STRUCT, "expected 'struct' in struct type expression");
  expect(Token::LBRACE, "expected '{' in struct type expression");
  auto propertyList = parsePropertyList();
  expect(Token::RBRACE, "expected '}' in struct type expression");
  return std::make_unique<StructTypeExpr>(std::move(propertyList));
}

unique_ptr<PropertyList> Parser::Parser::parsePropertyList() {
  std::vector<Property> properties;
  while (curTok->type != Token::RBRACE) {
    properties.push_back(parseProperty());
  }
  return std::make_unique<PropertyList>(std::move(properties));
}

Property Parser::Parser::parseProperty() {
  Property prop;
  prop.pub = false;
  prop.mut = false;
  if (curTok->type == Token::PUB) {
    eat();
    prop.pub = true;
  }
  if (curTok->type == Token::MUT) {
    eat();
    prop.mut = true;
  }
  prop.type = parseTypeExpr();
  prop.names = parseIdentList();
  return prop;
}

unique_ptr<InterfaceTypeExpr> Parser::Parser::parseInterfaceTypeExpr() {
  expect(Token::INTERFACE, "expected 'interface' in interface type expression");
  expect(Token::LBRACE, "expected '{' in interface type expression");

  auto methods = parseFieldList();

  expect(Token::RBRACE, "expected '}' in interface type expression");

  return std::make_unique<InterfaceTypeExpr>(std::move(methods));
}

unique_ptr<FieldList> Parser::Parser::parseFieldList() {
  std::vector<Field> fields;
  while (curTok->type != Token::RBRACE) {
    fields.push_back(parseField());
  }
  return std::make_unique<FieldList>(std::move(fields));
}

Field Parser::Parser::parseField() {
  Field field;

  field.name = expect(Token::IDENT, "expected identifier in field")->value;
  expect(Token::LPAREN, "expected '(' before field param list");
  field.params = parseParamList();
  expect(Token::RPAREN, "expected ')' at end of field param list");

  field.returnType = std::make_unique<PrimitiveTypeExpr>(Token::VOID);
  if (curTok->type == Token::ARROW) {
    eat();
    field.returnType = parseTypeExpr();
  }

  return field;
}

unique_ptr<Expr> Parser::Parser::parsePrimitiveTypeExpr() {
  Token::TokenType ty = expectRange(Token::types_begin, Token::types_end, "expected a type in primitive type expression")->type;
  if (curTok->type != Token::ASTERISK) return std::make_unique<PrimitiveTypeExpr>(ty);

  auto ptrTy = std::make_unique<PointerTypeExpr>(std::make_unique<PrimitiveTypeExpr>(ty));
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
  switch (curTok->type) {
    case Token::IDENT:
      return parseIdentExpr();
    case Token::INT:
    case Token::FLOAT:
    case Token::STRING_LIT:
      return parseBasicLit();
    default:
      break;
  }
  return nullptr;
}

unique_ptr<BasicLitExpr> Parser::Parser::parseBasicLit() {
  auto tok = expectRange(Token::basic_lit_begin, Token::basic_lit_end, "expected literal");
  return std::make_unique<BasicLitExpr>(tok->type, tok->value);
}

unique_ptr<Expr> Parser::Parser::parseCallExpr(const unique_ptr<Expr> &expr) {
  return nullptr;
}