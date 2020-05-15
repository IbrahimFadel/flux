#include "code_generation.h"

#include <llvm/IR/IRBuilder.h>
#include <llvm/IR/Module.h>
#include <llvm/IR/Value.h>
#include <llvm/IR/LLVMContext.h>
#include <llvm/Support/TargetSelect.h>
#include <llvm-c/Target.h>
#include <llvm/Support/raw_ostream.h>
#include <llvm/Support/FileSystem.h>
#include <llvm/IR/Verifier.h>

// using namespace llvm;

static llvm::LLVMContext context;
static llvm::Module *TheModule = new llvm::Module("example", context);
static llvm::IRBuilder<> Builder(context);
static std::map<std::string, llvm::Value *> NamedValues;

void generate_llvm_ir(std::vector<Node *> nodes)
{
  LLVMInitializeNativeTarget();

  llvm::raw_ostream *os = &llvm::outs();
  llvm::StringRef oname = "llvm_ir";
  std::error_code EC;
  llvm::raw_fd_ostream *out = new llvm::raw_fd_ostream(oname, EC);

  for (auto &node : nodes)
  {
    llvm::Value *v;
    llvm::Function *prototype;
    llvm::Function *function_body;
    llvm::Function *finished_function;

    switch (node->type)
    {
    case Node_Types::ConstantDeclarationNode:
    {
      v = std::get<Constant_Declaration_Node *>(node->constant_declaration_node)->code_gen();
      break;
    }
    case Node_Types::FunctionDeclarationNode:
    {
      prototype = std::get<Function_Declaration_Node *>(node->function_declaration_node)->code_gen_prototype();
      function_body = std::get<Function_Declaration_Node *>(node->function_declaration_node)->code_gen_function_body(prototype);
      prototype->print(*os);
      prototype->print(*out);
      break;
    }
    default:
      break;
    }
  }
}

llvm::Function *Function_Declaration_Node::code_gen_prototype()
{
  std::vector<llvm::Type *> Doubles(parameters.size(), llvm::Type::getDoubleTy(context));

  llvm::FunctionType *FT = llvm::FunctionType::get(llvm::Type::getDoubleTy(context), Doubles, false);

  llvm::Function *F = llvm::Function::Create(FT, llvm::Function::ExternalLinkage, name, TheModule);

  unsigned Idx = 0;
  std::map<std::string, Variable_Types>::iterator it;
  std::vector<std::string> names;
  for (it = parameters.begin(); it != parameters.end(); it++)
  {
    names.push_back(it->first);
  }
  for (auto &Arg : F->args())
  {
    Arg.setName(names[Idx]);
  }

  return F;
}

llvm::Function *Function_Declaration_Node::code_gen_function_body(llvm::Function *proto)
{
  llvm::Function *TheFunction = TheModule->getFunction(proto->getName());

  if (!TheFunction)
    TheFunction = code_gen_prototype();
  if (!TheFunction)
    return nullptr;
  if (!TheFunction->empty())
  {
    std::cerr << "Error" << std::endl;
    return nullptr;
  }

  llvm::BasicBlock *BB = llvm::BasicBlock::Create(context, "entry", TheFunction);
  Builder.SetInsertPoint(BB);

  NamedValues.clear();
  for (auto &Arg : TheFunction->args())
  {
    NamedValues[Arg.getName()] = &Arg;
  }

  // if(llvm::Value *RetValue = then.nodes)
  Builder.CreateRetVoid();

  llvm::verifyFunction(*TheFunction);

  // llvm::verify

  return TheFunction;
}

llvm::Function *Function_Declaration_Node::code_gen_finished(llvm::Function *Body)
{
  // if(llvm::Value *RetValue = Body->co)
}

llvm::Value *Constant_Declaration_Node::code_gen()
{
  switch (type)
  {
  case Variable_Types::FloatType:
  {
    float expression_value = evaluate_float_expression(std::move(expression));
    return llvm::ConstantFP::get(context, llvm::APFloat(expression_value));
    break;
  }
  default:
    return llvm::ConstantFP::get(context, llvm::APFloat(2.5));
    break;
  }
}

float evaluate_float_expression(std::unique_ptr<Expression_Node> expression)
{
  return 1.0;
}