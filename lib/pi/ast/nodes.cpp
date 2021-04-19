#include "nodes.h"

using namespace ssc;

bool ASTFunctionDefinition::getPub() { return pub; }
std::vector<Parameter> ASTFunctionDefinition::getParameters() { return parameters; }
std::string ASTFunctionDefinition::getReturnType() { return returnType; }
const std::vector<unique_ptr<ASTNode>> &ASTFunctionDefinition::getThen() { return then; }
void ASTFunctionDefinition::ASTFunctionDefinition::setMutable(std::string name, llvm::Value *val) { mutables[name] = val; }
llvm::Value *ASTFunctionDefinition::getMutable(std::string name) { return mutables[name]; }
void ASTFunctionDefinition::setConstant(std::string name, llvm::Value *val) { constants[name] = val; }
llvm::Value *ASTFunctionDefinition::getConstant(std::string name) { return constants[name]; }