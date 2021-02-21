// #include "ast.h"

// using namespace ssc;

// void FunctionDeclaration::setVariable(std::string name, llvm::Value *v) { variables[name] = v; }
// llvm::Value *FunctionDeclaration::getVariable(std::string name) { return variables[name]; }
// std::string FunctionDeclaration::getName() { return name; };
// std::map<std::string, std::string> FunctionDeclaration::getParams() { return params; };
// std::string FunctionDeclaration::getReturnType() { return returnType; };
// CodeBlock *FunctionDeclaration::getThen() { return then.get(); };

// std::string ImportStatement::getPath() { return path; };

// std::map<std::string, unique_ptr<Expression>> StructValueExpression::getProperties() { return std::move(properties); }
// std::vector<std::string> StructValueExpression::getPropertyInsertionOrder() { return propertyInsertionOrder; };

// std::string VariableReferenceExpression::getName() { return name; };

// TokenType BinaryOperationExpression::getOp() { return op; };
// unique_ptr<Expression> &BinaryOperationExpression::getLHS() { return lhs; };
// unique_ptr<Expression> &BinaryOperationExpression::getRHS() { return rhs; }

// double NumberExpression::getValue() { return value; }
// std::string StringLiteralExpression::getValue() { return value; }

// Expression *IndexAccessedExpression::getExpression() { return expr.get(); };
// Expression *IndexAccessedExpression::getIndex() { return index.get(); }

// Expression *UnaryPrefixOperationExpression::getValue() { return value.get(); }
// TokenType UnaryPrefixOperationExpression::getOp() { return op; }

// std::vector<unique_ptr<Node>> &CodeBlock::getNodes() { return nodes; }

// bool VariableDeclaration::getIsStruct() { return isStruct; }
// std::string VariableDeclaration::getName() { return name; }
// std::string VariableDeclaration::getType() { return type; }
// Expression *VariableDeclaration::getValue() { return value.get(); }

// std::string StructTypeExpression::getName() { return name; }
// std::map<std::string, std::string> StructTypeExpression::getProperties() { return properties; }
// std::vector<std::string> StructTypeExpression::getPropertyInsertionOrder() { return propertyInsertionOrder; }

// std::vector<unique_ptr<Expression>> IfStatement::getConditions() { return conditions; }
// std::vector<TokenType> IfStatement::getConditionSeparators() { return conditionSeparators; }
// CodeBlock *IfStatement::getThen() { return then.get(); }

// Expression *ReturnStatement::getValue() { return value.get(); }

// std::string FunctionCallExpression::getName() { return name; }
// const std::vector<unique_ptr<Expression>> &FunctionCallExpression::getParams() { return params; }