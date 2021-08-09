#include "dump.h"

std::string Parser::ASTDump::toString() {
  std::string astStrRepr = "";
  for (const auto &node : ast) {
    astStrRepr += node->toString();
  }
  return astStrRepr;
}

std::string Parser::FnDecl::toString() {
  std::string repr;
  repr += "FnDecl: {\n";
  if (receiver) {
    repr += receiver->toString();
  }
  repr += "Name: " + name + "\n";
  repr += type->toString();
  repr += "Body: {\n";
  repr += body->toString();
  repr += "}\n";
  repr += "}\n";
  return repr;
}

std::string Parser::FnReceiver::toString() {
  return "";
}

std::string Parser::FnType::toString() {
  std::string repr;
  repr += "Fn Type: {\n";
  repr += paramList->toString();
  repr += "Return Type: " + returnType->toString() + "\n";
  repr += "}\n";
  return repr;
}

std::string Parser::ParamList::toString() {
  std::string repr;
  repr += "Param List: {\n";
  for (auto const &param : params) {
    if (param.mut) {
      repr += "mut ";
    }
    repr += param.type->toString() + " ";
    repr += param.name + "\n";
  }
  repr += "}\n";
  return repr;
}

std::string Parser::BinaryExpr::toString() {
  return "";
}

std::string Parser::PrimitiveTypeExpr::toString() {
  std::string repr;
  repr += Token::tokens[type];
  return repr;
}

std::string Parser::PointerTypeExpr::toString() {
  return "";
}

std::string Parser::BasicLitExpr::toString() {
  std::string repr;
  repr += "Basic Lit Expr: {\n";
  repr += "Type: " + Token::tokens[type] + '\n';
  repr += "Value: " + value + '\n';
  repr += "}\n";
  return repr;
}

std::string Parser::BlockStmt::toString() {
  std::string repr;
  for (auto const &node : list) {
    repr += node->toString();
  }
  return repr;
}

std::string Parser::ReturnStmt::toString() {
  std::string repr;
  repr += "Return: {\n";
  repr += expr->toString();
  repr += "}\n";
  return repr;
}

std::string Parser::VarDecl::toString() {
  std::string repr;
  repr += "Var Decl: {\n";
  repr += "Mut: " + std::to_string(mut) + "\n";
  repr += "Type: " + type->toString() + "\n";
  repr += "Names: ";
  for (int i = 0; i < names.size(); i++) {
    repr += names[i];
    if (i < names.size() - 1) {
      repr += ", ";
    } else {
      repr += "\n";
    }
  }
  repr += "Values: {\n";
  for (auto const &val : values) {
    repr += val->toString();
  }
  repr += "}\n";
  repr += "}\n";
  return repr;
}

std::string Parser::NullExpr::toString() {
  std::string repr;
  return repr;
}