#include "dump.h"

std::string Parser::astToString(const unique_ptr<AST> &ast) {
  std::string repr;

  repr += "------- Types -------\n";
  for (auto const &ty : ast->types) {
    repr += ty->toString().c_str();
  }
  repr += "----- Variables -----\n";
  for (auto const &v : ast->variables) {
    repr += v->toString().c_str();
  }
  repr += "----- Functions -----\n";
  for (auto const &fn : ast->functions) {
    repr += fn->toString().c_str();
  }
  repr += "---------------------\n";
  return repr;
}

std::string Parser::FnDecl::toString() {
  std::string repr;
  repr += "Fn Decl: {\n";
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
  std::string repr;
  repr += "Fn Receiver: {\n";
  repr += "Type: " + type->toString();
  repr += "Name: " + name->toString();
  repr += "}\n";
  return repr;
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
    repr += names[i]->toString();
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
  return "Null\n";
}

std::string Parser::VoidExpr::toString() {
  return "Void\n";
}

std::string Parser::IdentExpr::toString() {
  return "Ident Expr: " + value + "\n";
}

std::string Parser::BinaryExpr::toString() {
  std::string repr;
  repr += "Binary Expr: {\n";
  repr += "Left: {\n";
  repr += x->toString();
  repr += "}\n";
  repr += "Operator: " + Token::tokens[op] + "\n";
  repr += "Right: {\n";
  repr += y->toString();
  repr += "}\n";
  repr += "}\n";
  return repr;
}

std::string Parser::TypeDecl::toString() {
  std::string repr;
  repr += "Type Declaration: {\n";
  repr += "Name: " + name + "\n";
  repr += "Type: {\n";
  repr += type->toString();
  repr += "}\n";
  repr += "}\n";
  return repr;
}

std::string Parser::InterfaceTypeExpr::toString() {
  std::string repr;
  repr += "Interface Type: {\n";
  repr += "Methods: {\n";
  repr += methods->toString();
  repr += "}\n";
  repr += "}\n";
  return repr;
}

std::string Parser::FieldList::toString() {
  std::string repr;
  repr += "Field List: {\n";
  for (int i = 0; i < fields.size(); i++) {
    repr += "Field " + std::to_string(i + 1) + " {\n";
    repr += "Name: " + fields[i].name + "\n";
    repr += "Params: {\n";
    repr += fields[i].params->toString();
    repr += "}\n";
    repr += "Return Type: {\n";
    repr += fields[i].returnType->toString() + "\n";
    repr += "}\n";
  }
  repr += "}\n";
  return repr;
}

std::string Parser::StructTypeExpr::toString() {
  std::string repr;
  repr += "Struct Type: {\n";
  repr += "Properties: {\n";
  repr += properties->toString();
  repr += "}\n";
  repr += "}\n";
  return repr;
}

std::string Parser::PropertyList::toString() {
  std::string repr;
  repr += "Property List: {\n";

  for (int i = 0; i < properties.size(); i++) {
    repr += "Property " + std::to_string(i + 1) + " {\n";
    repr += "Pub: " + std::to_string(properties[i].pub) + "\n";
    repr += "Mut: " + std::to_string(properties[i].mut) + "\n";
    repr += "Type: {\n";
    repr += properties[i].type->toString() + "\n";
    repr += "}\n";
    repr += "Names: {\n";
    for (int j = 0; j < properties[i].names.size(); j++) {
      repr += properties[i].names[j]->toString();
      if (i < properties[i].names.size() - 1) {
        repr += ", ";
      }
    }
    repr += "}\n";
    repr += "}\n";
  }

  repr += "}\n";
  return repr;
}
