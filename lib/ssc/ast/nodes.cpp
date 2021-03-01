#include "nodes.h"

using namespace ssc;

bool ASTFunctionDeclaration::getPub() { return pub; }
std::string ASTFunctionDeclaration::getName() { return name; }
std::vector<Parameter> ASTFunctionDeclaration::getParameters() { return parameters; }
std::string ASTFunctionDeclaration::getReturnType() { return returnType; }
const std::vector<unique_ptr<ASTNode>> &ASTFunctionDeclaration::getThen() { return then; }
void ASTFunctionDeclaration::ASTFunctionDeclaration::setMutable(std::string name, llvm::Value *val) { mutables[name] = val; }
llvm::Value *ASTFunctionDeclaration::getMutable(std::string name) { return mutables[name]; }
void ASTFunctionDeclaration::setConstant(std::string name, llvm::Value *val) { constants[name] = val; }
llvm::Value *ASTFunctionDeclaration::getConstant(std::string name) { return constants[name]; }